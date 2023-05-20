//! Ethereum Beacon Client
#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod functions;
pub mod impls;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
	dispatch::DispatchResult,
	log,
	pallet_prelude::OptionQuery,
	traits::{Get, UnixTime},
	transactional, BoundedVec,
};
use frame_system::ensure_signed;
use primitives::{
	fast_aggregate_verify, verify_merkle_branch, verify_receipt_proof, BeaconHeader, BlsError,
	CompactBeaconState, CompactExecutionHeader, ExecutionHeaderState, FinalizedHeaderState,
	ForkData, ForkVersion, ForkVersions, Mode, PublicKeyPrepared, SigningData,
};
use snowbridge_core::{Message, RingBufferMap, Verifier};
use sp_core::H256;
use sp_std::prelude::*;
pub use weights::WeightInfo;

use snowbridge_core::Proof;

use functions::{
	compute_epoch, compute_period, decompress_sync_committee_bits, sync_committee_sum,
};
use types::{
	CheckpointUpdate, ExecutionHeaderBuffer, ExecutionHeaderUpdate, FinalizedHeaderUpdate,
	SyncAggregate, SyncCommittee, SyncCommitteePrepared, SyncCommitteeUpdate, SyncCommitteesBuffer,
};

pub use pallet::*;

pub use config::SLOTS_PER_HISTORICAL_ROOT;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TimeProvider: UnixTime;
		#[pallet::constant]
		type ForkVersions: Get<ForkVersions>;
		#[pallet::constant]
		type WeakSubjectivityPeriodSeconds: Get<u64>;
		/// Maximum size of finalized header slot cache
		#[pallet::constant]
		type MaxFinalizedHeaderSlotsCacheSize: Get<u32>;
		/// Maximum finalized headers
		#[pallet::constant]
		type MaxFinalizedHeadersToKeep: Get<u32>;
		/// Maximum execution headers
		#[pallet::constant]
		type MaxExecutionHeadersToKeep: Get<u32>;
		/// Maximum sync committees
		#[pallet::constant]
		type MaxSyncCommitteesToKeep: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		BeaconHeaderImported { block_hash: H256, slot: u64 },
		ExecutionHeaderImported { block_hash: H256, block_number: u64 },
		SyncCommitteeUpdated { period: u64 },
	}

	#[pallet::error]
	#[cfg_attr(test, derive(PartialEq))]
	pub enum Error<T> {
		AncientHeader,
		SkippedSyncCommitteePeriod,
		SyncCommitteeMissing,
		Unknown,
		SyncCommitteeParticipantsNotSupermajority,
		InvalidHeaderMerkleProof,
		InvalidSyncCommitteeMerkleProof,
		InvalidExecutionHeaderProof,
		InvalidAncestryMerkleProof,
		InvalidBlockRootsRootMerkleProof,
		InvalidHash,
		InvalidSyncCommitteeBits,
		SignatureVerificationFailed,
		NoBranchExpected,
		HeaderNotFinalized,
		MissingHeader,
		MissingFinalityHeader,
		InvalidProof,
		InvalidBlockRootAtSlot,
		DecodeFailed,
		BlockBodyHashTreeRootFailed,
		BlockRootsHashTreeRootFailed,
		HeaderHashTreeRootFailed,
		SyncCommitteeHashTreeRootFailed,
		SigningRootHashTreeRootFailed,
		ForkDataHashTreeRootFailed,
		ExecutionHeaderNotLatest,
		UnexpectedHeaderSlotPosition,
		ExpectedFinalizedHeaderNotStored,
		BridgeBlocked,
		InvalidSyncCommitteeUpdateWithGap,
		InvalidSyncCommitteeUpdateWithDuplication,
		InvalidSignatureSlot,
		InvalidAttestedHeaderSlot,
		DuplicateFinalizedHeaderUpdate,
		InvalidFinalizedPeriodUpdate,
		InvalidExecutionHeaderUpdate,
		FinalizedBeaconHeaderSlotsExceeded,
		ExecutionHeaderMappingFailed,
		BLSPreparePublicKeysFailed,
		BLSVerificationFailed(BlsError),
	}

	/// A map of finalized headers to beacon state
	#[pallet::storage]
	pub(super) type FinalizedBeaconState<T: Config> =
		StorageMap<_, Identity, H256, CompactBeaconState, OptionQuery>;

	/// A cache of slot numbers for finalized headers that have been recently imported.
	/// Is used by the offchain relayer to produce ancestry proofs for execution headers.
	#[pallet::storage]
	pub(super) type FinalizedHeaderSlotsCache<T: Config> =
		StorageValue<_, BoundedVec<u64, T::MaxFinalizedHeadersToKeep>, ValueQuery>;

	#[pallet::storage]
	pub(super) type ExecutionHeaders<T: Config> =
		StorageMap<_, Identity, H256, CompactExecutionHeader, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators_root)]
	pub(super) type ValidatorsRoot<T: Config> = StorageValue<_, H256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_finalized_header)]
	pub(super) type LatestFinalizedHeader<T: Config> =
		StorageValue<_, FinalizedHeaderState, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_execution_header)]
	pub(super) type LatestExecutionHeader<T: Config> =
		StorageValue<_, ExecutionHeaderState, ValueQuery>;

	#[pallet::storage]
	pub(super) type LatestSyncCommitteePeriod<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type Blocked<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	pub(super) type SyncCommittees<T: Config> =
		StorageMap<_, Identity, u64, SyncCommitteePrepared, OptionQuery>;

	/// Index storage for execution header
	#[pallet::storage]
	pub(crate) type ExecutionHeaderIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Intermediate storage for execution header mapping
	#[pallet::storage]
	pub(crate) type ExecutionHeaderMapping<T: Config> =
		StorageMap<_, Identity, u32, H256, ValueQuery>;

	/// Index storage for sync committee ring buffer
	#[pallet::storage]
	pub(crate) type SyncCommitteesIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Intermediate storage for sync committee mapping
	#[pallet::storage]
	pub(crate) type SyncCommitteesMapping<T: Config> =
		StorageMap<_, Identity, u32, u64, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::force_mode())]
		#[transactional]
		pub fn force_mode(origin: OriginFor<T>, mode: Mode) -> DispatchResult {
			ensure_root(origin)?;

			match mode {
				Mode::Blocked => <Blocked<T>>::set(true),
				Mode::Active => <Blocked<T>>::set(false),
			}

			log::info!(target: "ethereum-beacon-client","💫 syncing bridge from governance provided checkpoint.");

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::force_checkpoint())]
		#[transactional]
		pub fn force_checkpoint(origin: OriginFor<T>, update: CheckpointUpdate) -> DispatchResult {
			ensure_root(origin)?;

			if let Err(err) = Self::process_checkpoint_update(&update) {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Sync committee period update failed with error {:?}",
					err
				);
				return Err(err)
			}

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Sync committee period update for slot {} succeeded.",
				update.header.slot
			);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::sync_committee_period_update())]
		#[transactional]
		pub fn sync_committee_period_update(
			origin: OriginFor<T>,
			update: SyncCommitteeUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Self::check_bridge_blocked_state()?;

			let sync_committee_period = compute_period(update.attested_header.slot);
			log::info!(
				target: "ethereum-beacon-client",
				"💫 Received sync committee update for period {}. Applying update",
				sync_committee_period
			);

			if let Err(err) = Self::process_sync_committee_period_update(&update) {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Sync committee period update failed with error {:?}",
					err
				);
				return Err(err)
			}

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Sync committee period update for period {} succeeded.",
				sync_committee_period
			);

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::import_finalized_header())]
		#[transactional]
		pub fn import_finalized_header(
			origin: OriginFor<T>,
			finalized_header_update: FinalizedHeaderUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Self::check_bridge_blocked_state()?;

			let slot = finalized_header_update.finalized_header.slot;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Received finalized header for slot {}.",
				slot
			);

			if let Err(err) = Self::process_finalized_header_update(&finalized_header_update) {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Finalized header update failed with error {:?}",
					err
				);
				return Err(err)
			}

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Stored finalized beacon header at slot {}.",
				slot
			);

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::import_execution_header())]
		#[transactional]
		pub fn import_execution_header(
			origin: OriginFor<T>,
			update: ExecutionHeaderUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Self::check_bridge_blocked_state()?;

			let slot = update.header.slot;
			let block_hash = update.execution_header.block_hash;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Received header update for slot {}.",
				slot
			);

			if let Err(err) = Self::process_execution_header_update(&update) {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Header update failed with error {:?}",
					err
				);
				return Err(err)
			}

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Stored execution header {} at beacon slot {}.",
				block_hash,
				slot
			);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn process_checkpoint_update(update: &CheckpointUpdate) -> DispatchResult {
			Self::verify_sync_committee(
				&update.current_sync_committee,
				&update.current_sync_committee_branch,
				update.header.state_root,
				config::CURRENT_SYNC_COMMITTEE_SUBTREE_INDEX,
				config::CURRENT_SYNC_COMMITTEE_DEPTH,
			)?;
			let period = compute_period(update.header.slot);

			let header_root: H256 = update
				.header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			// Verify update.block_roots_root
			ensure!(
				verify_merkle_branch(
					update.block_roots_root,
					&update.block_roots_branch,
					config::BLOCK_ROOTS_SUBTREE_INDEX,
					config::BLOCK_ROOTS_DEPTH,
					update.header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidBlockRootsRootMerkleProof
			);

			Self::store_sync_committee(period, &update.current_sync_committee)?;
			Self::store_validators_root(update.validators_root);
			Self::store_finalized_header(
				header_root,
				update.header,
				update.block_roots_root,
				Some(update.import_time),
			)?;

			Ok(())
		}

		fn process_sync_committee_period_update(update: &SyncCommitteeUpdate) -> DispatchResult {
			Self::verify_weak_subjectivity()?;
			Self::verify_attested_header(
				&update.attested_header,
				&update.sync_aggregate,
				update.signature_slot,
			)?;
			let finalized_header_root = Self::verify_finalized_header(
				&update.attested_header,
				&update.finalized_header,
				&update.finality_branch,
			)?;

			Self::verify_sync_committee(
				&update.next_sync_committee,
				&update.next_sync_committee_branch,
				update.attested_header.state_root,
				config::NEXT_SYNC_COMMITTEE_SUBTREE_INDEX,
				config::NEXT_SYNC_COMMITTEE_DEPTH,
			)?;

			let current_period = compute_period(update.attested_header.slot);
			let latest_committee_period = <LatestSyncCommitteePeriod<T>>::get();
			log::trace!(
				target: "ethereum-beacon-client",
				"💫 latest committee period is: {}, attested_header period is: {}",
				latest_committee_period,
				current_period,
			);

			let next_period = current_period + 1;
			ensure!(
				!<SyncCommitteesBuffer<T>>::contains_key(next_period),
				Error::<T>::InvalidSyncCommitteeUpdateWithDuplication
			);
			ensure!(
				(next_period == latest_committee_period + 1),
				Error::<T>::InvalidSyncCommitteeUpdateWithGap
			);

			// Verify update.block_roots_root
			ensure!(
				verify_merkle_branch(
					update.block_roots_root,
					&update.block_roots_branch,
					config::BLOCK_ROOTS_SUBTREE_INDEX,
					config::BLOCK_ROOTS_DEPTH,
					update.finalized_header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidBlockRootsRootMerkleProof
			);

			Self::store_sync_committee(next_period, &update.next_sync_committee)?;
			Self::store_finalized_header(
				finalized_header_root,
				update.finalized_header,
				update.block_roots_root,
				None,
			)?;

			Ok(())
		}

		fn process_finalized_header_update(update: &FinalizedHeaderUpdate) -> DispatchResult {
			Self::verify_weak_subjectivity()?;
			Self::verify_attested_header(
				&update.attested_header,
				&update.sync_aggregate,
				update.signature_slot,
			)?;
			ensure!(
				update.finalized_header.slot > Self::latest_finalized_header().beacon_slot,
				Error::<T>::DuplicateFinalizedHeaderUpdate
			);
			let finalized_header_root = Self::verify_finalized_header(
				&update.attested_header,
				&update.finalized_header,
				&update.finality_branch,
			)?;

			let last_finalized_period = compute_period(Self::latest_finalized_header().beacon_slot);
			let current_period = compute_period(update.attested_header.slot);
			ensure!(
				(current_period == last_finalized_period ||
					current_period == last_finalized_period + 1),
				Error::<T>::InvalidFinalizedPeriodUpdate
			);

			// Verify update.block_roots_root
			ensure!(
				verify_merkle_branch(
					update.block_roots_root,
					&update.block_roots_branch,
					config::BLOCK_ROOTS_SUBTREE_INDEX,
					config::BLOCK_ROOTS_DEPTH,
					update.finalized_header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidBlockRootsRootMerkleProof
			);

			Self::store_finalized_header(
				finalized_header_root,
				update.finalized_header,
				update.block_roots_root,
				None,
			)?;

			Ok(())
		}

		fn process_execution_header_update(update: &ExecutionHeaderUpdate) -> DispatchResult {
			ensure!(
				update.header.slot <= Self::latest_finalized_header().beacon_slot,
				Error::<T>::HeaderNotFinalized
			);

			ensure!(
				update.execution_header.block_number > Self::latest_execution_header().block_number,
				Error::<T>::InvalidExecutionHeaderUpdate
			);

			let execution_header_root: H256 = update
				.execution_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::BlockBodyHashTreeRootFailed)?;

			ensure!(
				verify_merkle_branch(
					execution_header_root,
					&update.execution_branch,
					config::EXECUTION_HEADER_SUBTREE_INDEX,
					config::EXECUTION_HEADER_DEPTH,
					update.header.body_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidExecutionHeaderProof
			);

			let block_root: H256 = update
				.header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			match &update.ancestry_proof {
				Some(proof) => {
					Self::verify_ancestry_proof(
						block_root,
						update.header.slot,
						&proof.header_branch,
						proof.finalized_block_root,
					)?;
				},
				None => {
					// If the ancestry proof is not provided, we expect this header to be a
					// finalized header. We need to check that the header hash matches the finalized
					// header root at the expected slot.
					let state = <FinalizedBeaconState<T>>::get(block_root)
						.ok_or(Error::<T>::ExpectedFinalizedHeaderNotStored)?;
					if update.header.slot != state.slot {
						return Err(Error::<T>::ExpectedFinalizedHeaderNotStored.into())
					}
				},
			}

			Self::store_execution_header(
				update.execution_header.block_hash,
				update.execution_header.clone().into(),
				update.header.slot,
				block_root,
			);

			Ok(())
		}

		// Verify that `block_root` is an ancestor of `finalized_block_root`
		fn verify_ancestry_proof(
			block_root: H256,
			block_slot: u64,
			block_root_proof: &[H256],
			finalized_block_root: H256,
		) -> DispatchResult {
			let state = <FinalizedBeaconState<T>>::get(finalized_block_root)
				.ok_or(Error::<T>::ExpectedFinalizedHeaderNotStored)?;

			let index_in_array = block_slot % (SLOTS_PER_HISTORICAL_ROOT as u64);
			let leaf_index = (SLOTS_PER_HISTORICAL_ROOT as u64) + index_in_array;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Depth: {} leaf_index: {}", config::BLOCK_ROOT_AT_INDEX_DEPTH, leaf_index
			);

			ensure!(
				verify_merkle_branch(
					block_root,
					&block_root_proof,
					leaf_index as usize,
					config::BLOCK_ROOT_AT_INDEX_DEPTH,
					state.block_roots_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidAncestryMerkleProof
			);

			Ok(())
		}

		fn check_bridge_blocked_state() -> DispatchResult {
			if <Blocked<T>>::get() {
				return Err(Error::<T>::BridgeBlocked.into())
			}
			Ok(())
		}

		// Weak subjectivity check
		pub(super) fn verify_weak_subjectivity() -> DispatchResult {
			let import_time = Self::latest_finalized_header().import_time;
			let weak_subjectivity_period_check =
				import_time + T::WeakSubjectivityPeriodSeconds::get();
			let time: u64 = T::TimeProvider::now().as_secs();
			log::info!(
				target: "ethereum-beacon-client",
				"💫 Checking weak subjectivity period. Current time is :{:?} Weak subjectivity period check: {:?}.",
				time,
				weak_subjectivity_period_check
			);
			if time > weak_subjectivity_period_check {
				log::info!(target: "ethereum-beacon-client","💫 Weak subjectivity period exceeded, blocking bridge.",);
				<Blocked<T>>::set(true);
				// FIXME: reverting the transaction will revert the state change!
				return Err(Error::<T>::BridgeBlocked.into())
			}

			Ok(())
		}

		/// Verify that a supermajority of the sync committee signed the attested beacon header
		pub(super) fn verify_attested_header(
			attested_header: &BeaconHeader,
			sync_aggregate: &SyncAggregate,
			signature_slot: u64,
		) -> DispatchResult {
			// Verify sync committee has sufficient participants
			let participation = decompress_sync_committee_bits(sync_aggregate.sync_committee_bits);
			Self::sync_committee_participation_is_supermajority(&participation)?;

			// Verify update does not skip a sync committee period
			ensure!(signature_slot > attested_header.slot, Error::<T>::InvalidSignatureSlot);

			// Verify sync committee aggregate signature
			let current_period = compute_period(attested_header.slot);
			let sync_committee = Self::sync_committee_for_period(current_period)?;
			let absent_pubkeys = Self::find_pubkeys(&participation, &sync_committee.pubkeys, false);
			let signing_root =
				Self::signing_root(attested_header, Self::validators_root(), signature_slot)?;
			fast_aggregate_verify(
				&sync_committee.aggregate_pubkey,
				&absent_pubkeys,
				signing_root,
				&sync_aggregate.sync_committee_signature,
			)
			.map_err(|e| Error::<T>::BLSVerificationFailed(e))?;

			Ok(())
		}

		pub(super) fn verify_finalized_header(
			attested_header: &BeaconHeader,
			finalized_header: &BeaconHeader,
			finality_branch: &[H256],
		) -> Result<H256, DispatchError> {
			ensure!(
				attested_header.slot >= finalized_header.slot,
				Error::<T>::InvalidAttestedHeaderSlot
			);

			let finalized_block_root: H256 = finalized_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			ensure!(
				verify_merkle_branch(
					finalized_block_root,
					finality_branch,
					config::FINALIZED_ROOT_SUBTREE_INDEX,
					config::FINALIZED_ROOT_DEPTH,
					attested_header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidHeaderMerkleProof
			);

			Ok(finalized_block_root)
		}

		pub(super) fn compute_signing_root(
			beacon_header: &BeaconHeader,
			domain: H256,
		) -> Result<H256, DispatchError> {
			let beacon_header_root = beacon_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			let hash_root = SigningData { object_root: beacon_header_root, domain }
				.hash_tree_root()
				.map_err(|_| Error::<T>::SigningRootHashTreeRootFailed)?;

			Ok(hash_root)
		}

		fn verify_sync_committee(
			sync_committee: &SyncCommittee,
			sync_committee_branch: &[H256],
			header_state_root: H256,
			index: usize,
			depth: usize,
		) -> DispatchResult {
			let sync_committee_root = sync_committee
				.hash_tree_root()
				.map_err(|_| Error::<T>::SyncCommitteeHashTreeRootFailed)?;

			ensure!(
				verify_merkle_branch(
					sync_committee_root,
					sync_committee_branch,
					index,
					depth,
					header_state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

			Ok(())
		}

		pub(crate) fn store_sync_committee(
			period: u64,
			sync_committee: &SyncCommittee,
		) -> DispatchResult {
			let prepare_sync_committee: SyncCommitteePrepared =
				sync_committee.try_into().map_err(|_| <Error<T>>::BLSPreparePublicKeysFailed)?;
			<SyncCommitteesBuffer<T>>::insert(period, prepare_sync_committee);

			<LatestSyncCommitteePeriod<T>>::set(period);

			log::debug!(
				target: "ethereum-beacon-client",
				"💫 Updated latest sync committee period stored to {}.",
				period
			);

			Self::deposit_event(Event::SyncCommitteeUpdated { period });
			Ok(())
		}

		fn store_finalized_header(
			header_root: H256,
			header: BeaconHeader,
			block_roots_root: H256,
			last_import_time: Option<u64>,
		) -> DispatchResult {
			let slot = header.slot;
			let import_time = last_import_time.unwrap_or_else(|| T::TimeProvider::now().as_secs());

			<FinalizedBeaconState<T>>::insert(
				header_root,
				CompactBeaconState { slot: header.slot, block_roots_root },
			);

			<LatestFinalizedHeader<T>>::set(FinalizedHeaderState {
				beacon_block_root: header_root,
				beacon_slot: slot,
				import_time,
			});

			// Add the slot of the most recently finalized header to the slot cache
			<FinalizedHeaderSlotsCache<T>>::mutate(|slots| {
				if slots.len() as u32 == T::MaxFinalizedHeaderSlotsCacheSize::get() {
					slots.remove(0);
				}
				slots.try_push(header.slot).expect("checked above; qed");
			});

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Updated latest finalized block root {} at slot {}.",
				header_root,
				slot
			);

			Self::deposit_event(Event::BeaconHeaderImported { block_hash: header_root, slot });

			Ok(())
		}

		pub(crate) fn store_execution_header(
			block_hash: H256,
			header: CompactExecutionHeader,
			beacon_slot: u64,
			beacon_block_root: H256,
		) {
			let block_number = header.block_number;

			<ExecutionHeaderBuffer<T>>::insert(block_hash, header);

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Updated latest execution block at {} to number {}.",
				block_hash,
				block_number
			);

			LatestExecutionHeader::<T>::mutate(|s| {
				s.beacon_block_root = beacon_block_root;
				s.beacon_slot = beacon_slot;
				s.block_hash = block_hash;
				s.block_number = block_number;
			});

			Self::deposit_event(Event::ExecutionHeaderImported { block_hash, block_number });
		}

		fn store_validators_root(validators_root: H256) {
			<ValidatorsRoot<T>>::set(validators_root);
		}

		/// Return the domain for the domain_type and fork_version.
		pub(super) fn compute_domain(
			domain_type: Vec<u8>,
			fork_version: ForkVersion,
			genesis_validators_root: H256,
		) -> Result<H256, DispatchError> {
			let fork_data_root =
				Self::compute_fork_data_root(fork_version, genesis_validators_root)?;

			let mut domain = [0u8; 32];
			domain[0..4].copy_from_slice(&(domain_type));
			domain[4..32].copy_from_slice(&(fork_data_root.0[..28]));

			Ok(domain.into())
		}

		fn compute_fork_data_root(
			current_version: ForkVersion,
			genesis_validators_root: H256,
		) -> Result<H256, DispatchError> {
			let hash_root = ForkData {
				current_version,
				genesis_validators_root: genesis_validators_root.into(),
			}
			.hash_tree_root()
			.map_err(|_| Error::<T>::ForkDataHashTreeRootFailed)?;

			Ok(hash_root)
		}

		pub(super) fn sync_committee_participation_is_supermajority(
			sync_committee_bits: &[u8],
		) -> DispatchResult {
			let sync_committee_sum = sync_committee_sum(sync_committee_bits);
			ensure!(
				((sync_committee_sum * 3) as usize) >= sync_committee_bits.len() * 2,
				Error::<T>::SyncCommitteeParticipantsNotSupermajority
			);

			Ok(())
		}

		pub(super) fn sync_committee_for_period(
			period: u64,
		) -> Result<SyncCommitteePrepared, DispatchError> {
			<SyncCommitteesBuffer<T>>::get(period).ok_or(Error::<T>::SyncCommitteeMissing.into())
		}

		pub(super) fn compute_fork_version(epoch: u64) -> ForkVersion {
			let fork_versions = T::ForkVersions::get();

			if epoch >= fork_versions.capella.epoch {
				return fork_versions.capella.version
			}
			if epoch >= fork_versions.bellatrix.epoch {
				return fork_versions.bellatrix.version
			}
			if epoch >= fork_versions.altair.epoch {
				return fork_versions.altair.version
			}

			fork_versions.genesis.version
		}

		pub fn find_pubkeys(
			sync_committee_bits: &[u8],
			sync_committee_pubkeys: &[PublicKeyPrepared],
			participant: bool,
		) -> Vec<PublicKeyPrepared> {
			let mut pubkeys: Vec<PublicKeyPrepared> = Vec::new();
			for (bit, pubkey) in sync_committee_bits.iter().zip(sync_committee_pubkeys.iter()) {
				if *bit == u8::from(participant) {
					pubkeys.push(*pubkey);
				}
			}
			pubkeys
		}

		// Calculate signing root for BeaconHeader
		pub fn signing_root(
			header: &BeaconHeader,
			validators_root: H256,
			signature_slot: u64,
		) -> Result<H256, DispatchError> {
			let fork_version = Self::compute_fork_version(compute_epoch(
				signature_slot,
				config::SLOTS_PER_EPOCH as u64,
			));
			let domain_type = config::DOMAIN_SYNC_COMMITTEE.to_vec();
			// Domains are used for for seeds, for signatures, and for selecting aggregators.
			let domain = Self::compute_domain(domain_type, fork_version, validators_root)?;
			// Hash tree root of SigningData - object root + domain
			let signing_root = Self::compute_signing_root(header, domain)?;
			Ok(signing_root)
		}
	}
}
