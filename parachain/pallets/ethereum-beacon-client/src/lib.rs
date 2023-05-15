//! # Ethereum Beacon Client
#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[cfg(not(feature = "minimal"))]
mod tests_mainnet;

#[cfg(test)]
#[cfg(feature = "minimal")]
mod tests_minimal;

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
	fast_aggregate_verify, verify_merkle_proof, verify_receipt_proof, BeaconHeader, BlsError,
	CompactExecutionHeader, ExecutionHeaderState, FinalizedHeaderState, ForkData, ForkVersion,
	ForkVersions, PublicKeyPrepared, Signature, SigningData,
};
use snowbridge_core::{Message, RingBufferMap, RingBufferMapImpl, Verifier};
use sp_core::H256;
use sp_std::prelude::*;
pub use weights::WeightInfo;

use snowbridge_core::Proof;
use snowbridge_ethereum::{Header as EthereumHeader, Log, Receipt};
use sp_core::U256;

pub use pallet::*;

pub use config::{SLOTS_PER_HISTORICAL_ROOT, SYNC_COMMITTEE_BITS_SIZE, SYNC_COMMITTEE_SIZE};

pub type CheckPointUpdate = primitives::CheckPointUpdate<SYNC_COMMITTEE_SIZE>;
pub type ExecutionHeaderUpdate =
	primitives::ExecutionHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>;
pub type SyncCommitteeUpdate =
	primitives::SyncCommitteeUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>;
pub type FinalizedHeaderUpdate =
	primitives::FinalizedHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>;
pub type SyncCommittee = primitives::SyncCommittee<SYNC_COMMITTEE_SIZE>;
pub type SyncCommitteePrepared = primitives::SyncCommitteePrepared<SYNC_COMMITTEE_SIZE>;

fn decompress_sync_committee_bits(
	input: [u8; SYNC_COMMITTEE_BITS_SIZE],
) -> [u8; SYNC_COMMITTEE_SIZE] {
	primitives::decompress_sync_committee_bits::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(
		input,
	)
}

/// ExecutionHeader ring buffer implementation
pub(crate) type ExecutionHeaderBuffer<T> = RingBufferMapImpl<
	u32,
	<T as Config>::MaxExecutionHeadersToKeep,
	ExecutionHeaderIndex<T>,
	ExecutionHeaderMapping<T>,
	ExecutionHeaders<T>,
	OptionQuery,
>;

/// Sync committee ring buffer implementation
pub(crate) type SyncCommitteesBuffer<T> = RingBufferMapImpl<
	u32,
	<T as Config>::MaxSyncCommitteesToKeep,
	SyncCommitteesIndex<T>,
	SyncCommitteesMapping<T>,
	SyncCommittees<T>,
	OptionQuery,
>;

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
		InvalidHash,
		InvalidSyncCommitteeBits,
		SignatureVerificationFailed,
		NoBranchExpected,
		HeaderNotFinalized,
		MissingHeader,
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
		InvalidSyncCommitteeHeaderUpdate,
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

	#[pallet::storage]
	pub(super) type FinalizedBeaconHeaders<T: Config> =
		StorageMap<_, Identity, H256, BeaconHeader, OptionQuery>;

	#[pallet::storage]
	pub(super) type FinalizedBeaconHeaderStates<T: Config> =
		StorageValue<_, BoundedVec<FinalizedHeaderState, T::MaxFinalizedHeadersToKeep>, ValueQuery>;

	#[pallet::storage]
	pub(super) type FinalizedBeaconHeadersBlockRoot<T: Config> =
		StorageMap<_, Identity, H256, H256, ValueQuery>;

	#[pallet::storage]
	pub(super) type ExecutionHeaders<T: Config> =
		StorageMap<_, Identity, H256, CompactExecutionHeader, OptionQuery>;

	#[pallet::storage]
	pub(super) type ValidatorsRoot<T: Config> = StorageValue<_, H256, ValueQuery>;

	#[pallet::storage]
	pub(super) type LatestFinalizedHeaderState<T: Config> =
		StorageValue<_, FinalizedHeaderState, ValueQuery>;

	#[pallet::storage]
	pub(super) type LatestExecutionHeaderState<T: Config> =
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
		#[pallet::weight(T::WeightInfo::sync_committee_period_update())]
		#[transactional]
		pub fn sync_committee_period_update(
			origin: OriginFor<T>,
			update: SyncCommitteeUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Self::check_bridge_blocked_state()?;

			let sync_committee_period = update.sync_committee_period;
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

		#[pallet::call_index(1)]
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

			if let Err(err) = Self::process_finalized_header(finalized_header_update) {
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

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::import_execution_header())]
		#[transactional]
		pub fn import_execution_header(
			origin: OriginFor<T>,
			update: ExecutionHeaderUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			Self::check_bridge_blocked_state()?;

			let slot = update.beacon_header.slot;
			let block_hash = update.execution_header.block_hash;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Received header update for slot {}.",
				slot
			);

			if let Err(err) = Self::process_header(update) {
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

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::activate_bridge())]
		#[transactional]
		pub fn activate_bridge(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			<Blocked<T>>::set(false);

			log::info!(target: "ethereum-beacon-client","💫 syncing bridge from governance provided checkpoint.");

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::deactivate_bridge())]
		#[transactional]
		pub fn deactivate_bridge(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			<Blocked<T>>::set(true);

			log::info!(target: "ethereum-beacon-client","💫 syncing bridge from governance provided checkpoint.");

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::check_point_update())]
		#[transactional]
		pub fn check_point_update(
			origin: OriginFor<T>,
			update: CheckPointUpdate,
		) -> DispatchResult {
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
	}

	impl<T: Config> Pallet<T> {
		pub fn process_checkpoint_update(update: &CheckPointUpdate) -> DispatchResult {
			Self::verify_sync_committee(
				&update.current_sync_committee,
				&update.current_sync_committee_branch,
				update.header.state_root,
				config::CURRENT_SYNC_COMMITTEE_INDEX,
			)?;

			let period = Self::compute_current_sync_period(update.header.slot);

			let block_root: H256 = update
				.header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			Self::store_sync_committee(period, &update.current_sync_committee)?;
			Self::store_validators_root(update.validators_root);
			Self::store_finalized_header(block_root, update.header, Some(update.import_time))?;

			Ok(())
		}

		fn process_sync_committee_period_update(update: &SyncCommitteeUpdate) -> DispatchResult {
			ensure!(
				update.signature_slot > update.attested_header.slot &&
					update.attested_header.slot >= update.finalized_header.slot,
				Error::<T>::InvalidSyncCommitteeHeaderUpdate
			);
			let participation =
				decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
			Self::sync_committee_participation_is_supermajority(&participation)?;
			Self::verify_sync_committee(
				&update.next_sync_committee,
				&update.next_sync_committee_branch,
				update.attested_header.state_root,
				config::NEXT_SYNC_COMMITTEE_INDEX,
			)?;

			let block_root: H256 = update
				.finalized_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			Self::verify_header(
				block_root,
				&update.finality_branch,
				update.attested_header.state_root,
				config::FINALIZED_ROOT_INDEX,
			)?;

			let current_period = Self::compute_current_sync_period(update.attested_header.slot);
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

			let validators_root = <ValidatorsRoot<T>>::get();
			let sync_committee = Self::sync_committee_for_period(current_period)?;
			Self::verify_signed_header(
				&participation,
				&update.sync_aggregate.sync_committee_signature,
				&sync_committee,
				update.attested_header,
				validators_root,
				update.signature_slot,
			)?;
			ensure!(
				update.block_roots_branch.len() == config::BLOCK_ROOTS_DEPTH &&
					verify_merkle_proof(
						update.block_roots_root,
						&update.block_roots_branch,
						config::BLOCK_ROOTS_INDEX,
						update.finalized_header.state_root
					),
				Error::<T>::InvalidAncestryMerkleProof
			);

			Self::store_block_root(update.block_roots_root, block_root);
			Self::store_sync_committee(next_period, &update.next_sync_committee)?;
			Self::store_finalized_header(block_root, update.finalized_header, None)?;

			Ok(())
		}

		fn process_finalized_header(update: FinalizedHeaderUpdate) -> DispatchResult {
			let last_finalized_header = <LatestFinalizedHeaderState<T>>::get();
			ensure!(
				update.signature_slot > update.attested_header.slot,
				Error::<T>::InvalidSignatureSlot
			);
			ensure!(
				update.attested_header.slot >= update.finalized_header.slot,
				Error::<T>::InvalidAttestedHeaderSlot
			);
			ensure!(
				update.finalized_header.slot > last_finalized_header.beacon_slot,
				Error::<T>::DuplicateFinalizedHeaderUpdate
			);

			let import_time = last_finalized_header.import_time;
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
				return Err(Error::<T>::BridgeBlocked.into())
			}

			let participation =
				decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
			Self::sync_committee_participation_is_supermajority(&participation)?;

			let block_root: H256 = update
				.finalized_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			Self::verify_header(
				block_root,
				&update.finality_branch,
				update.attested_header.state_root,
				config::FINALIZED_ROOT_INDEX,
			)?;

			let last_finalized_period =
				Self::compute_current_sync_period(last_finalized_header.beacon_slot);
			let current_period = Self::compute_current_sync_period(update.attested_header.slot);
			ensure!(
				(current_period == last_finalized_period ||
					current_period == last_finalized_period + 1),
				Error::<T>::InvalidFinalizedPeriodUpdate
			);

			let validators_root = <ValidatorsRoot<T>>::get();
			let sync_committee = Self::sync_committee_for_period(current_period)?;
			Self::verify_signed_header(
				&participation,
				&update.sync_aggregate.sync_committee_signature,
				&sync_committee,
				update.attested_header,
				validators_root,
				update.signature_slot,
			)?;

			ensure!(
				update.block_roots_branch.len() == config::BLOCK_ROOTS_DEPTH &&
					verify_merkle_proof(
						update.block_roots_root,
						&update.block_roots_branch,
						config::BLOCK_ROOTS_INDEX,
						update.finalized_header.state_root
					),
				Error::<T>::InvalidAncestryMerkleProof
			);

			Self::store_block_root(update.block_roots_root, block_root);

			Self::store_finalized_header(block_root, update.finalized_header, None)?;

			Ok(())
		}

		fn store_block_root(block_roots_hash: H256, finalized_header_block_root: H256) {
			<FinalizedBeaconHeadersBlockRoot<T>>::insert(
				finalized_header_block_root,
				block_roots_hash,
			);
		}

		fn process_header(update: ExecutionHeaderUpdate) -> DispatchResult {
			let last_finalized_header = <LatestFinalizedHeaderState<T>>::get();
			let latest_finalized_header_slot = last_finalized_header.beacon_slot;
			let block_slot = update.beacon_header.slot;
			ensure!(block_slot <= latest_finalized_header_slot, Error::<T>::HeaderNotFinalized);

			let execution_header_state = <LatestExecutionHeaderState<T>>::get();
			ensure!(
				update.execution_header.block_number > execution_header_state.block_number,
				Error::<T>::InvalidExecutionHeaderUpdate
			);

			let execution_root: H256 = update
				.execution_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::BlockBodyHashTreeRootFailed)?;

			ensure!(
				update.execution_branch.len() == config::EXECUTION_HEADER_DEPTH &&
					verify_merkle_proof(
						execution_root,
						&update.execution_branch,
						config::EXECUTION_HEADER_INDEX,
						update.beacon_header.body_root
					),
				Error::<T>::InvalidExecutionHeaderProof
			);

			let beacon_block_root: H256 = update
				.beacon_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			Self::ancestry_proof(
				update.block_root_branch,
				block_slot,
				beacon_block_root,
				update.block_root_branch_header_root,
			)?;

			let current_period = Self::compute_current_sync_period(update.beacon_header.slot);
			let sync_committee = Self::sync_committee_for_period(current_period)?;

			let validators_root = <ValidatorsRoot<T>>::get();
			let participation =
				decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);

			Self::verify_signed_header(
				&participation,
				&update.sync_aggregate.sync_committee_signature,
				&sync_committee,
				update.beacon_header,
				validators_root,
				update.signature_slot,
			)?;

			Self::store_execution_header(
				update.execution_header.block_hash,
				update.execution_header.into(),
				block_slot,
				beacon_block_root,
			);

			Ok(())
		}

		fn ancestry_proof(
			block_root_proof: Vec<H256>,
			block_slot: u64,
			beacon_block_root: H256,
			finalized_header_root: H256,
		) -> DispatchResult {
			// If the block root proof is empty, we know that we expect this header to be a
			// finalized header. We need to check that the header hash matches the finalized header
			// root at the expected slot.
			if block_root_proof.is_empty() {
				let stored_finalized_header = <FinalizedBeaconHeaders<T>>::get(beacon_block_root);
				if stored_finalized_header.is_none() {
					log::error!(
						target: "ethereum-beacon-client",
						"💫 Finalized block root {} slot {} for ancestry proof (for a finalized header) not found.", beacon_block_root, block_slot
					);
					return Err(Error::<T>::ExpectedFinalizedHeaderNotStored.into())
				}

				let header = stored_finalized_header.unwrap();
				if header.slot != block_slot {
					log::error!(
						target: "ethereum-beacon-client",
						"💫 Finalized block root {} slot {} does not match expected slot {}.", beacon_block_root, block_slot, header.slot
					);
					return Err(Error::<T>::UnexpectedHeaderSlotPosition.into())
				}

				return Ok(())
			}

			let finalized_block_root_hash =
				<FinalizedBeaconHeadersBlockRoot<T>>::get(finalized_header_root);

			if finalized_block_root_hash.is_zero() {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Finalized block root {} slot {} for ancestry proof not found.", beacon_block_root, block_slot
				);
				return Err(Error::<T>::ExpectedFinalizedHeaderNotStored.into())
			}

			let index_in_array = block_slot % (SLOTS_PER_HISTORICAL_ROOT as u64);
			let leaf_index = (SLOTS_PER_HISTORICAL_ROOT) + index_in_array as usize;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Depth: {} leaf_index: {}", config::BLOCK_ROOT_AT_INDEX_PROOF_DEPTH, leaf_index
			);

			ensure!(
				block_root_proof.len() == config::BLOCK_ROOT_AT_INDEX_PROOF_DEPTH &&
					verify_merkle_proof(
						beacon_block_root,
						&block_root_proof,
						leaf_index,
						finalized_block_root_hash
					),
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

		pub(super) fn verify_signed_header(
			sync_committee_bits: &[u8],
			sync_committee_signature: &Signature,
			sync_committee: &SyncCommitteePrepared,
			header: BeaconHeader,
			validators_root: H256,
			signature_slot: u64,
		) -> DispatchResult {
			// Gathers milagro pubkeys absent to participate
			let absent_pubkeys =
				Self::find_pubkeys(sync_committee_bits, &sync_committee.pubkeys, false);

			// Get signing root for BeaconHeader
			let signing_root = Self::signing_root(header, validators_root, signature_slot)?;

			// Verify sync committee aggregate signature.
			fast_aggregate_verify(
				&sync_committee.aggregate_pubkey,
				&absent_pubkeys,
				signing_root,
				sync_committee_signature,
			)
			.map_err(|e| Error::<T>::BLSVerificationFailed(e))?;

			Ok(())
		}

		pub(super) fn compute_epoch_at_slot(signature_slot: u64, slots_per_epoch: u64) -> u64 {
			signature_slot / slots_per_epoch
		}

		pub(super) fn compute_signing_root(
			beacon_header: BeaconHeader,
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
		) -> DispatchResult {
			let sync_committee_root = sync_committee
				.hash_tree_root()
				.map_err(|_| Error::<T>::SyncCommitteeHashTreeRootFailed)?;

			ensure!(
				sync_committee_branch.len() == config::SYNC_COMMITTEE_DEPTH &&
					verify_merkle_proof(
						sync_committee_root,
						sync_committee_branch,
						index,
						header_state_root
					),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

			Ok(())
		}

		fn verify_header(
			block_root: H256,
			proof_branch: &[H256],
			attested_header_state_root: H256,
			index: usize,
		) -> DispatchResult {
			ensure!(
				proof_branch.len() == config::FINALIZED_ROOT_DEPTH &&
					verify_merkle_proof(
						block_root,
						proof_branch,
						index,
						attested_header_state_root
					),
				Error::<T>::InvalidHeaderMerkleProof
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
			block_root: H256,
			header: BeaconHeader,
			last_import_time: Option<u64>,
		) -> DispatchResult {
			let slot = header.slot;
			let import_time = last_import_time.unwrap_or_else(|| T::TimeProvider::now().as_secs());

			let finalized_header = FinalizedHeaderState {
				beacon_block_root: block_root,
				beacon_slot: slot,
				import_time,
			};

			<FinalizedBeaconHeaders<T>>::insert(block_root, header);
			LatestFinalizedHeaderState::<T>::set(finalized_header);
			Self::add_finalized_header_state(finalized_header)?;

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Updated latest finalized block root {} at slot {}.",
				block_root,
				slot
			);

			Self::deposit_event(Event::BeaconHeaderImported { block_hash: block_root, slot });

			Ok(())
		}

		pub(super) fn add_finalized_header_state(
			finalized_header_state: FinalizedHeaderState,
		) -> DispatchResult {
			<FinalizedBeaconHeaderStates<T>>::try_mutate(|b_vec| {
				if b_vec.len() as u32 == T::MaxFinalizedHeadersToKeep::get() {
					let oldest = b_vec.remove(0);
					// Removing corresponding finalized header data of popped slot
					// as that data will not be used by relayer anyway.
					<FinalizedBeaconHeadersBlockRoot<T>>::remove(oldest.beacon_block_root);
					<FinalizedBeaconHeaders<T>>::remove(oldest.beacon_block_root);
				}
				b_vec.try_push(finalized_header_state)
			})
			.map_err(|_| <Error<T>>::FinalizedBeaconHeaderSlotsExceeded)?;

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

			LatestExecutionHeaderState::<T>::mutate(|s| {
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

		/// Sums the bit vector of sync committee particpation.
		///
		/// # Examples
		///
		/// let sync_committee_bits = vec![0, 1, 0, 1, 1, 1];
		/// ensure!(get_sync_committee_sum(sync_committee_bits), 4);
		pub(super) fn get_sync_committee_sum(sync_committee_bits: &[u8]) -> u32 {
			sync_committee_bits.iter().fold(0, |acc: u32, x| acc + *x as u32)
		}

		pub(super) fn compute_current_sync_period(slot: u64) -> u64 {
			(slot as usize / config::SLOTS_PER_EPOCH / config::EPOCHS_PER_SYNC_COMMITTEE_PERIOD)
				as u64
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
			let sync_committee_sum = Self::get_sync_committee_sum(sync_committee_bits);
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

		// Verifies that the receipt encoded in proof.data is included
		// in the block given by proof.block_hash. Inclusion is only
		// recognized if the block has been finalized.
		fn verify_receipt_inclusion(
			receipts_root: H256,
			proof: &Proof,
		) -> Result<Receipt, DispatchError> {
			let result = verify_receipt_proof(receipts_root, &proof.data.1)
				.ok_or(Error::<T>::InvalidProof)?;

			match result {
				Ok(receipt) => Ok(receipt),
				Err(err) => {
					log::trace!(
						target: "ethereum-beacon-client",
						"💫 Failed to decode transaction receipt: {}",
						err
					);
					Err(Error::<T>::InvalidProof.into())
				},
			}
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
			header: BeaconHeader,
			validators_root: H256,
			signature_slot: u64,
		) -> Result<H256, DispatchError> {
			let fork_version = Self::compute_fork_version(Self::compute_epoch_at_slot(
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

	impl<T: Config> Verifier for Pallet<T> {
		/// Verify a message by verifying the existence of the corresponding
		/// Ethereum log in a block. Returns the log if successful.
		fn verify(message: &Message) -> Result<Log, DispatchError> {
			log::info!(
				target: "ethereum-beacon-client",
				"💫 Verifying message with block hash {}",
				message.proof.block_hash,
			);

			let header = <ExecutionHeaderBuffer<T>>::get(message.proof.block_hash)
				.ok_or(Error::<T>::MissingHeader)?;

			let receipt = match Self::verify_receipt_inclusion(header.receipts_root, &message.proof)
			{
				Ok(receipt) => receipt,
				Err(err) => {
					log::error!(
						target: "ethereum-beacon-client",
						"💫 Verify receipt inclusion failed for block {}: {:?}",
						message.proof.block_hash,
						err
					);
					return Err(err)
				},
			};

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Verified receipt inclusion for transaction at index {} in block {}",
				message.proof.tx_index, message.proof.block_hash,
			);

			let log = match rlp::decode(&message.data) {
				Ok(log) => log,
				Err(err) => {
					log::error!(
						target: "ethereum-beacon-client",
						"💫 RLP log decoded failed {}: {:?}",
						message.proof.block_hash,
						err
					);
					return Err(Error::<T>::DecodeFailed.into())
				},
			};

			if !receipt.contains_log(&log) {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Event log not found in receipt for transaction at index {} in block {}",
					message.proof.tx_index, message.proof.block_hash,
				);
				return Err(Error::<T>::InvalidProof.into())
			}

			log::info!(
				target: "ethereum-beacon-client",
				"💫 Receipt verification successful for {}",
				message.proof.block_hash,
			);

			Ok(log)
		}

		// Empty implementation, not necessary for the beacon client,
		// but needs to be declared to implement Verifier interface.
		fn initialize_storage(
			_headers: Vec<EthereumHeader>,
			_initial_difficulty: U256,
			_descendants_until_final: u8,
		) -> Result<(), &'static str> {
			Ok(())
		}
	}
}
