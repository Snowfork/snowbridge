//! # Ethereum 2 Light Client Verifier
//!
//! This module implements the `Verifier` interface. Other modules should reference
//! this module using the `Verifier` type and perform verification using `Verifier::verify`.
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
mod merklization;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, log, transactional};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_core::hashing::sha2_256;

type Root = H256;
type Domain = H256;
type ValidatorIndex = u64;

const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 22;
const CURRENT_SYNC_COMMITTEE_DEPTH: u64 = 5;

/// Beacon block header as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BeaconBlockHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	pub slot: u64,
	// The index of the validator that proposed the block.
	pub proposer_index: ValidatorIndex,
	// The block root of the parent block, forming a block chain.
	pub parent_root: Root,
	// The hash root of the post state of running the state transition through this block.
	pub state_root: Root,
	// The hash root of the Eth1 block
	pub body_root: Root,
}

/// Sync committee as it is stored in the runtime storage.
#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo
)]
pub struct SyncCommittee {
	pub pubkeys: Vec<Vec<u8>>,
	pub aggregate_pubkey: Vec<u8>,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct SyncCommitteeBranch {
	pub branch: Vec<Vec<u8>>,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct LightClientInitialSync {
	pub header: BeaconBlockHeader,
	pub current_sync_committee: SyncCommittee,
	pub current_sync_committee_branch: SyncCommitteeBranch,
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(
	Clone,
	Default,
	Encode,
	Decode,
	PartialEq,
	RuntimeDebug,
	TypeInfo,
)]
pub struct SigningData {
	pub object_root: Root,
	pub domain: Domain,
}

pub use pallet::*;

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
	}

	#[pallet::event]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
		AncientHeader,
		SkippedSyncCommitteePeriod,
		Unknown,
		InsufficientSyncCommitteeParticipants,
		InvalidSyncCommiteeSignature,
		InvalidMerkleProof,
		InvalidSignature,
		InvalidSignaturePoint,
		InvalidAggregatePublicKeys,
		InvalidHash,
		SignatureVerificationFailed,
		NoBranchExpected,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	pub(super) type FinalizedHeaders<T: Config> = StorageMap<_, Identity, H256, BeaconBlockHeader, OptionQuery>;

	#[pallet::storage]
	pub(super) type FinalizedHeadersBySlot<T: Config> = StorageMap<_, Identity, u64, H256, OptionQuery>;

	/// Current sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type CurrentSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

	/// Next sync committee corresponding to the active header
	#[pallet::storage]
	pub(super) type NextSyncCommittee<T: Config> = StorageValue<_, SyncCommittee, ValueQuery>;

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
		pub fn initial_sync(
			origin: OriginFor<T>,
			initial_sync: LightClientInitialSync,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"Received update {:?}. Starting initial_sync",
				initial_sync
			);

			Self::process_initial_sync(initial_sync)
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_initial_sync(
			initial_sync: LightClientInitialSync,
		) -> DispatchResult {
			Self::verify_sync_committee(initial_sync.clone())?;

			Self::store_header(initial_sync.header);
			
			Ok(())
		}

		fn verify_sync_committee(initial_sync: LightClientInitialSync) -> DispatchResult {
			let sync_committee_root = merklization::hash_tree_root_sync_committee(initial_sync.current_sync_committee).map_err(|_| DispatchError::Other("Sync committee hash tree root failed"))?;

			let mut branch =  Vec::<H256>::new();

			for vec_branch in initial_sync.current_sync_committee_branch.branch.iter() {
				branch.push(H256::from_slice(vec_branch.as_slice()));
			}

			ensure!(
				Self::is_valid_merkle_branch(
					sync_committee_root.into(),
					branch,
					CURRENT_SYNC_COMMITTEE_DEPTH,
					CURRENT_SYNC_COMMITTEE_INDEX,
					initial_sync.header.state_root
				),
				Error::<T>::InvalidMerkleProof
			);

			Ok(())
		}

		fn store_header(header: BeaconBlockHeader) {
			<FinalizedHeaders<T>>::insert(header.body_root.clone(), header.clone());

			<FinalizedHeadersBySlot<T>>::insert(header.slot, header.body_root);
		}

		pub(super) fn is_valid_merkle_branch(
			leaf: H256,
			branch: Vec<H256>,
			depth: u64,
			index: u64,
			root: Root,
		) -> bool {
			let mut value = leaf;
			for i in 0..depth {
				if (index / (2u32.pow(i as u32) as u64) % 2) == 0 {
					// left node
					let mut data = [0u8; 64];
					data[0..32].copy_from_slice(&(value.0));
					data[32..64].copy_from_slice(&(branch[i as usize].0));
					value = sha2_256(&data).into();
				} else {
					let mut data = [0u8; 64]; // right node
					data[0..32].copy_from_slice(&(branch[i as usize].0));
					data[32..64].copy_from_slice(&(value.0));
					value = sha2_256(&data).into();
				}
			}
			return value == root;
		}
	}
}
