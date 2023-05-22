//! Ethereum Beacon Client
#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod functions;
pub mod impls;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(all(test, not(feature = "beacon-spec-mainnet")))]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use frame_support::{
	dispatch::DispatchResult, log, pallet_prelude::OptionQuery, traits::Get, transactional,
	BoundedVec,
};
use frame_system::ensure_signed;
use primitives::{
	fast_aggregate_verify, verify_merkle_branch, verify_receipt_proof, BeaconHeader, BlsError,
	CompactBeaconState, CompactExecutionHeader, ExecutionHeaderState, FinalizedHeaderState,
	ForkData, ForkVersion, ForkVersions, PublicKeyPrepared, SigningData,
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
	CheckpointUpdate, ExecutionHeaderBuffer, ExecutionHeaderUpdate, SyncCommittee,
	SyncCommitteePrepared, Update,
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
		#[pallet::constant]
		type ForkVersions: Get<ForkVersions>;
		/// Maximum number of execution headers to keep
		#[pallet::constant]
		type MaxExecutionHeadersToKeep: Get<u32>;
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
		NotRelevant,
		Unknown,
		NotBootstrapped,
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
		ExecutionHeaderAlreadyImported,
		FinalizedBeaconHeaderSlotsExceeded,
		ExecutionHeaderMappingFailed,
		BLSPreparePublicKeysFailed,
		BLSVerificationFailed(BlsError),
	}

	/// Latest imported finalized block root
	#[pallet::storage]
	#[pallet::getter(fn latest_finalized_block_root)]
	pub(super) type LatestFinalizedBlockRoot<T: Config> = StorageValue<_, H256, ValueQuery>;

	/// Beacon state by finalized block root
	#[pallet::storage]
	#[pallet::getter(fn finalized_beacon_state)]
	pub(super) type FinalizedBeaconState<T: Config> =
		StorageMap<_, Identity, H256, CompactBeaconState, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators_root)]
	pub(super) type ValidatorsRoot<T: Config> = StorageValue<_, H256, ValueQuery>;

	/// Sync committee for current period
	#[pallet::storage]
	pub(super) type CurrentSyncCommittee<T: Config> =
		StorageValue<_, SyncCommitteePrepared, ValueQuery>;

	/// Sync committee for next period
	#[pallet::storage]
	pub(super) type NextSyncCommittee<T: Config> =
		StorageValue<_, SyncCommitteePrepared, ValueQuery>;

	/// Latest imported execution header
	#[pallet::storage]
	#[pallet::getter(fn latest_execution_header)]
	pub(super) type LatestExecutionHeader<T: Config> =
		StorageValue<_, ExecutionHeaderState, ValueQuery>;

	/// Execution Headers
	#[pallet::storage]
	pub(super) type ExecutionHeaders<T: Config> =
		StorageMap<_, Identity, H256, CompactExecutionHeader, OptionQuery>;

	/// Execution Headers: Current position in ring buffer
	#[pallet::storage]
	pub(crate) type ExecutionHeaderIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Execution Headers: Mapping of ring buffer index to a pruning candidate
	#[pallet::storage]
	pub(crate) type ExecutionHeaderMapping<T: Config> =
		StorageMap<_, Identity, u32, H256, ValueQuery>;

	/// FIXME: Remove before using in production
	/// A cache of slot numbers for finalized headers that have been recently imported.
	/// Is used by the offchain relayer to produce ancestry proofs for execution headers.
	#[pallet::storage]
	pub(super) type FinalizedHeaderSlotsCache<T: Config> =
		StorageValue<_, BoundedVec<u64, ConstU32<{ config::SLOT_CACHE_SIZE }>>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::force_checkpoint())]
		#[transactional]
		pub fn force_checkpoint(origin: OriginFor<T>, update: CheckpointUpdate) -> DispatchResult {
			ensure_root(origin)?;
			Self::process_checkpoint_update(&update)?;
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({
			match update.next_sync_committee_update {
				None => T::WeightInfo::submit(),
				Some(_) => T::WeightInfo::submit_with_sync_committee(),
			}
		})]
		#[transactional]
		pub fn submit(origin: OriginFor<T>, update: Update) -> DispatchResult {
			ensure_signed(origin)?;
			Self::process_update(&update)?;
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::submit_execution_header())]
		#[transactional]
		pub fn submit_execution_header(
			origin: OriginFor<T>,
			update: ExecutionHeaderUpdate,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::process_execution_header_update(&update)?;
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn process_checkpoint_update(update: &CheckpointUpdate) -> DispatchResult {
			let sync_committee_root = update
				.current_sync_committee
				.hash_tree_root()
				.map_err(|_| Error::<T>::SyncCommitteeHashTreeRootFailed)?;

			ensure!(
				verify_merkle_branch(
					sync_committee_root,
					&update.current_sync_committee_branch,
					config::CURRENT_SYNC_COMMITTEE_SUBTREE_INDEX,
					config::CURRENT_SYNC_COMMITTEE_DEPTH,
					update.header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

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

			let sync_committee_prepared: SyncCommitteePrepared = (&update.current_sync_committee)
				.try_into()
				.map_err(|_| <Error<T>>::BLSPreparePublicKeysFailed)?;
			<CurrentSyncCommittee<T>>::set(sync_committee_prepared);

			Self::store_validators_root(update.validators_root);
			Self::store_finalized_header(header_root, update.header, update.block_roots_root)?;

			Ok(())
		}

		fn process_update(update: &Update) -> DispatchResult {
			// Verify update does not skip a sync committee period
			ensure!(
				update.signature_slot > update.attested_header.slot,
				Error::<T>::InvalidSignatureSlot
			);

			// Retrieve latest finalized state
			let latest_finalized_state =
				match Self::finalized_beacon_state(Self::latest_finalized_block_root()) {
					Some(finalized_beacon_state) => finalized_beacon_state,
					None => return Err(Error::<T>::NotBootstrapped.into()),
				};

			let store_period = compute_period(latest_finalized_state.slot);
			let signature_period = compute_period(update.signature_slot);
			if <NextSyncCommittee<T>>::exists() {
				ensure!(
					(store_period..=store_period + 1).contains(&signature_period),
					Error::<T>::SkippedSyncCommitteePeriod
				)
			} else {
				ensure!(signature_period == store_period, Error::<T>::SkippedSyncCommitteePeriod)
			}

			// Verify update is relevant
			let update_attested_period = compute_period(update.attested_header.slot);
			let update_has_next_sync_committee = !<NextSyncCommittee<T>>::exists() &&
				(update.next_sync_committee_update.is_some() &&
					update_attested_period == store_period);
			ensure!(
				update.attested_header.slot > store_period || update_has_next_sync_committee,
				Error::<T>::NotRelevant
			);

			// Verify sync committee has sufficient participants
			let participation =
				decompress_sync_committee_bits(update.sync_aggregate.sync_committee_bits);
			Self::sync_committee_participation_is_supermajority(&participation)?;

			// Verify sync committee aggregate signature
			let sync_committee = if signature_period == store_period {
				<CurrentSyncCommittee<T>>::get()
			} else {
				<NextSyncCommittee<T>>::get()
			};

			let absent_pubkeys = Self::find_pubkeys(&participation, &sync_committee.pubkeys, false);
			let signing_root = Self::signing_root(
				&update.attested_header,
				Self::validators_root(),
				update.signature_slot,
			)?;
			fast_aggregate_verify(
				&sync_committee.aggregate_pubkey,
				&absent_pubkeys,
				signing_root,
				&update.sync_aggregate.sync_committee_signature,
			)
			.map_err(|e| Error::<T>::BLSVerificationFailed(e))?;

			Self::process_finalized_header_update(update, store_period)?;

			if let Some(next_sync_committee_update) = &update.next_sync_committee_update {
				Self::process_next_sync_committee_update(
					update,
					signature_period,
					store_period,
					&next_sync_committee_update.next_sync_committee,
					&next_sync_committee_update.next_sync_committee_branch,
				)?;
			}

			Ok(())
		}

		fn process_finalized_header_update(update: &Update, store_period: u64) -> DispatchResult {
			ensure!(
				update.attested_header.slot >= update.finalized_header.slot,
				Error::<T>::InvalidAttestedHeaderSlot
			);

			let finalized_block_root: H256 = update
				.finalized_header
				.hash_tree_root()
				.map_err(|_| Error::<T>::HeaderHashTreeRootFailed)?;

			// Verify that the header is a finalized checkpoint
			ensure!(
				verify_merkle_branch(
					finalized_block_root,
					&update.finality_branch,
					config::FINALIZED_ROOT_SUBTREE_INDEX,
					config::FINALIZED_ROOT_DEPTH,
					update.attested_header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidHeaderMerkleProof
			);

			let attested_period = compute_period(update.attested_header.slot);
			ensure!(
				(store_period..=store_period + 1).contains(&attested_period),
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

			if update.finalized_header.slot > store_period {
				Self::store_finalized_header(
					finalized_block_root,
					update.finalized_header,
					update.block_roots_root,
				)?;
			}

			Ok(())
		}

		fn process_next_sync_committee_update(
			update: &Update,
			signature_period: u64,
			store_period: u64,
			next_sync_committee: &SyncCommittee,
			next_sync_committee_branch: &[H256],
		) -> DispatchResult {
			let update_finalized_period = compute_period(update.finalized_header.slot);

			let next_sync_committee_root = next_sync_committee
				.hash_tree_root()
				.map_err(|_| Error::<T>::SyncCommitteeHashTreeRootFailed)?;

			ensure!(
				verify_merkle_branch(
					next_sync_committee_root,
					&next_sync_committee_branch,
					config::NEXT_SYNC_COMMITTEE_SUBTREE_INDEX,
					config::NEXT_SYNC_COMMITTEE_DEPTH,
					update.attested_header.state_root
				)
				.is_some_and(|x| x),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

			let sync_committee_prepared: SyncCommitteePrepared = next_sync_committee
				.try_into()
				.map_err(|_| <Error<T>>::BLSPreparePublicKeysFailed)?;

			if !<NextSyncCommittee<T>>::exists() {
				<NextSyncCommittee<T>>::set(sync_committee_prepared);
				Self::deposit_event(Event::SyncCommitteeUpdated { period: signature_period });
			} else if update_finalized_period == store_period + 1 {
				<CurrentSyncCommittee<T>>::set(<NextSyncCommittee<T>>::get());
				<NextSyncCommittee<T>>::set(sync_committee_prepared);
				Self::deposit_event(Event::SyncCommitteeUpdated { period: signature_period });
			}

			Ok(())
		}

		fn process_execution_header_update(update: &ExecutionHeaderUpdate) -> DispatchResult {
			ensure!(
				update.execution_header.block_number > Self::latest_execution_header().block_number,
				Error::<T>::ExecutionHeaderAlreadyImported
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

			ensure!(block_slot < state.slot, Error::<T>::HeaderNotFinalized);

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

		fn store_finalized_header(
			header_root: H256,
			header: BeaconHeader,
			block_roots_root: H256,
		) -> DispatchResult {
			let slot = header.slot;

			<FinalizedBeaconState<T>>::insert(
				header_root,
				CompactBeaconState { slot: header.slot, block_roots_root },
			);
			<LatestFinalizedBlockRoot<T>>::set(header_root);

			// Add the slot of the most recently finalized header to the slot cache
			<FinalizedHeaderSlotsCache<T>>::mutate(|slots| {
				if slots.len() as u32 == config::SLOT_CACHE_SIZE {
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
