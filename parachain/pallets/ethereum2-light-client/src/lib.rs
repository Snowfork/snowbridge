//! # Ethereum 2 Light Client Verifier
//!
//! This module implements the `Verifier` interface. Other modules should reference
//! this module using the `Verifier` type and perform verification using `Verifier::verify`.
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, log, traits::Get, transactional};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use std::cmp;

pub use snowbridge_ethereum::Header as EthereumHeader;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#misc
/// The minimum number of validators that needs to sign update
const MIN_SYNC_COMMITTEE_PARTICIPANTS: u8 = 1;

const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u32 = 512;

const SECONDS_PER_SLOT: u32 = 12;

const SLOTS_PER_EPOCH: u32 = 32;

const SYNC_COMMITTEE_SIZE: u32 = 256;

const UPDATE_TIMEOUT: u32 = SLOTS_PER_EPOCH * EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

/// Beacon block header as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
// https://yeeth.github.io/BeaconChain.swift/Structs/BeaconBlockHeader.html#/s:11BeaconChain0A11BlockHeaderV9signature10Foundation4DataVvp
pub struct BeaconBlockHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	pub slot: u64,
	// The block root of the parent block, forming a block chain.
	pub parent_root: H256,
	// The hash root of the post state of running the state transition through this block.
	pub state_root: H256,
	// BLS Signature of the block by the block proposer, TODO type isn't right yet
	pub signature: String,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Root {
	// TODO: Add
}

/// Sync committee as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncCommittee {
	// TODO: Add
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	pub sync_committee_bits: Vec<H256> // TODO this isn't right
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Version {}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct LightClientUpdate {
	/// The beacon block header that is attested to by the sync committee
	pub attested_header: BeaconBlockHeader,
	///  Next sync committee corresponding to the active header
	pub next_sync_committee: SyncCommittee,
	/// Vector[Bytes32, floorlog2(NEXT_SYNC_COMMITTEE_INDEX)]
	pub next_sync_committee_branch: Vec<H256>,
	/// The finalized beacon block header attested to by Merkle branch
	pub finalized_header: Option<BeaconBlockHeader>,
	/// Vector[Bytes32, floorlog2(FINALIZED_ROOT_INDEX)]
	pub finality_branch: Vec<H256>,
	///  Sync committee aggregate signature
	pub sync_aggregate: SyncAggregate,
	///  Fork version for the aggregate signature
	pub pubfork_version: Version,
}

pub use frame_system::pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#constants
		/// Finalized root index - TODO not a useful comment, will elaborate as understanding grows
		#[pallet::constant]
		type FinalizedRootIndex: Get<u16>;
		/// Next sync committee index - TODO not a useful comment, will elaborate as understanding grows
		#[pallet::constant]
		type NextSyncCommitteeIndex: Get<u16>;
	}

	#[pallet::event]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
		AncientHeader,
		SkippedSyncCommitteePeriod,
		Unknown,
		InsufficientSyncCommitteeParticipants
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#lightclientstore
	/// Beacon block header that is finalized
	#[pallet::storage]
	pub(super) type FinalizedHeader<T: Config> = StorageValue<_, BeaconBlockHeader, ValueQuery>;

	/// Current sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type CurrentSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

	/// Next sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type NextSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

	/// Best available header to switch finalized head to if we see nothing else
	#[pallet::storage]
	pub(super) type BestValidUpdate<T: Config> =
		StorageValue<_, Option<LightClientUpdate>, ValueQuery>; // TODO: maybe I should use OptionQuery here?

	/// Most recent available reasonably-safe header
	#[pallet::storage]
	pub(super) type OptimisticHeader<T: Config> = StorageValue<_, BeaconBlockHeader, ValueQuery>;

	/// Max number of active participants in a sync committee (used to calculate safety threshold)
	#[pallet::storage]
	pub(super) type PreviousMaxActiveParticipants<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type CurrentMaxActiveParticipants<T: Config> = StorageValue<_, u64, ValueQuery>;

	// Would these also go into the store?
	// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#lightclientupdate

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		// genesis header goes header, maybe?
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn import_header(origin: OriginFor<T>, update: LightClientUpdate) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum2-light-client",
				"Received update {:?}. Starting validation",
				update
			);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn validate_light_client_update(
			update: LightClientUpdate,
			current_slot: u64,
			genesis_validators_root: Root,
		) -> DispatchResult {
			// Verify update slot is larger than slot of current best finalized header
			let active_header = Self::get_active_header(update.clone());
			let finalized_header = <FinalizedHeader<T>>::get();
			ensure!(
				(current_slot >= active_header.slot)
					&& (active_header.slot > finalized_header.slot),
				Error::<T>::AncientHeader
			);

			//Verify update does not skip a sync committee period
			let finalized_period = Self::compute_sync_committee_period(
				Self::compute_epoch_at_slot(finalized_header.slot),
			);
			let update_period = Self::compute_sync_committee_period(Self::compute_epoch_at_slot(
				active_header.slot,
			));
			ensure!(
				(update_period == finalized_period) || (update_period == finalized_period + 1),
				Error::<T>::SkippedSyncCommitteePeriod
			);

			// Verify that the `finalized_header`, if present, actually is the finalized header saved in the
			// state of the `attested header`.
			if update.finalized_header.is_some() {
				// update.finality_branch == [Bytes32() for _ in range(floorlog2(FINALIZED_ROOT_INDEX))]
				ensure!(true, Error::<T>::Unknown); // TODO
			} else {
				//  assert is_valid_merkle_branch(
				//	leaf=hash_tree_root(update.finalized_header),
				//	branch=update.finality_branch,
				//	depth=floorlog2(FINALIZED_ROOT_INDEX),
				//	index=get_subtree_index(FINALIZED_ROOT_INDEX),
				//	root=update.attested_header.state_root,
				ensure!(Self::is_valid_merkle_branch(), Error::<T>::Unknown);
			}

			// Verify update next sync committee if the update period incremented
			if update_period == finalized_period {
				let sync_committee = <CurrentSyncCommittee<T>>::get();
				//a ssert update.next_sync_committee_branch == [Bytes32() for _ in range(floorlog2(NEXT_SYNC_COMMITTEE_INDEX))]
				ensure!(true, Error::<T>::Unknown); // TODO
			} else {
				let sync_committee = <NextSyncCommittee<T>>::get();
				// assert is_valid_merkle_branch(
				//	leaf=hash_tree_root(update.next_sync_committee),
				//	branch=update.next_sync_committee_branch,
				//	depth=floorlog2(NEXT_SYNC_COMMITTEE_INDEX),
				//	index=get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX),
				//	root=active_header.state_root,
				ensure!(Self::is_valid_merkle_branch(), Error::<T>::Unknown);
			}
        
			let sync_aggregate = update.sync_aggregate;

			// TODO .len isn't right
			// assert sum(sync_aggregate.sync_committee_bits) >= MIN_SYNC_COMMITTEE_PARTICIPANTS
			ensure!(sync_aggregate.sync_committee_bits.len() >= MIN_SYNC_COMMITTEE_PARTICIPANTS as usize, Error::<T>::InsufficientSyncCommitteeParticipants);
		
			Ok(())
		}

		fn process_slot_for_light_client_store(current_slot: u64) {
			if current_slot % UPDATE_TIMEOUT as u64 == 0 {
				let curr_max_active_participants = <PreviousMaxActiveParticipants<T>>::get();
				<PreviousMaxActiveParticipants<T>>::put(curr_max_active_participants);
				<CurrentMaxActiveParticipants<T>>::put(0);
			}

			let finalized_header = <FinalizedHeader<T>>::get();
			let best_valid_update = <BestValidUpdate<T>>::get();

			if current_slot > (finalized_header.slot + UPDATE_TIMEOUT as u64)
				&& best_valid_update.is_some()
			{
				// Forced best update when the update timeout has elapsed
				Self::apply_light_client_update(best_valid_update.unwrap());
				<BestValidUpdate<T>>::kill(); // TODO does this set the value to None?
			}
		}

		fn apply_light_client_update(update: LightClientUpdate) {
			let active_header = Self::get_active_header(update.clone());

			let finalized_header = <FinalizedHeader<T>>::get();

			let finalized_period = Self::compute_sync_committee_period(
				Self::compute_epoch_at_slot(finalized_header.slot),
			);
			let update_period = Self::compute_sync_committee_period(Self::compute_epoch_at_slot(
				active_header.slot,
			));

			if update_period == (finalized_period + 1) {
				<CurrentSyncCommittee<T>>::put(<NextSyncCommittee<T>>::get());
				<NextSyncCommittee<T>>::put(update.next_sync_committee);
			}
			<FinalizedHeader<T>>::put(active_header.clone());

			let optimistic_header = <OptimisticHeader<T>>::get();

			let finalized_header = <FinalizedHeader<T>>::get();

			if finalized_header.slot > optimistic_header.slot {
				<OptimisticHeader<T>>::put(finalized_header.clone());
			}
		}

		fn compute_epoch_at_slot(slot: u64) -> u64 {
			1 // TODO
		}

		fn compute_sync_committee_period(epoch_at_slot: u64) -> u64 {
			1 // TODO
		}

		fn is_valid_merkle_branch() -> bool {
			true // TODO
		}

		//** Helper functions **//
		// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#helper-functions

		fn get_subtree_index() -> u64 {
			// return uint64(generalized_index % 2**(floorlog2(generalized_index)))
			1
		}

		fn get_active_header(update: LightClientUpdate) -> BeaconBlockHeader {
			// The "active header" is the header that the update is trying to convince us
			// to accept. If a finalized header is present, it's the finalized header,
			// otherwise it's the attested header
			match update.finalized_header {
				Some(finalized_header) => finalized_header,
				None => update.attested_header,
			}
		}

		fn get_safety_threshold(
			prev_max_active_participants: u64,
			curr_max_active_participants: u64,
		) -> u64 {
			cmp::max(prev_max_active_participants, curr_max_active_participants)
		}
	}
}
