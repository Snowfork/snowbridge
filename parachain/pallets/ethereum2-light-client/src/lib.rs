//! # Ethereum 2 Light Client Verifier
//!
//! This module implements the `Verifier` interface. Other modules should reference
//! this module using the `Verifier` type and perform verification using `Verifier::verify`.
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	log,
	traits::Get,
	transactional,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{convert::TryInto, prelude::*};

use snowbridge_core::{Message, Proof, Verifier};
use snowbridge_ethereum::{Log, Receipt, H256, U256,};
pub use snowbridge_ethereum::{
	Header as EthereumHeader,
};

pub use weights::WeightInfo;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#misc
/// The minimum number of validators
const MIN_SYNC_COMMITTEE_PARTICIPANTS: u8 = 1;
/// SLOTS_PER_EPOCH * EPOCHS_PER_SYNC_COMMITTEE_PERIOD in seconds	
const UPDATE_TIMEOUT: u64 = 8; // TODO update

/// Beacon block header as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BeaconBlockHeader {
    // TODO: Add
}

/// Sync committee as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncCommittee {
    // TODO: Add
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{pallet_prelude::*, BoundedVec};
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
        /// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
        // TODO: Add
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
    pub(super) type BestValidUpdate<T: Config> = StorageValue<_, BeaconBlockHeader, ValueQuery>;

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
		fn build(&self) {

		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn import_header(
			origin: OriginFor<T>,
			header: EthereumHeader,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum2-light-client",
				"Received header {}. Starting validation",
				header.number,
			);

			Ok(())
		}
	}
}