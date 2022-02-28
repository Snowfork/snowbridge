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

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, log, traits::Get, transactional};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_io::hashing::sha2_256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use ssz::Encode as SSZEncode;
use ssz_derive::{Decode as SSZDecode, Encode as SSZEncode};
use tree_hash_derive::TreeHash;
use std::cmp;
use milagro_bls::{AggregateSignature, AggregatePublicKey, Signature, PublicKey, AmclError};

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#misc
/// The minimum number of validators that needs to sign update
const MIN_SYNC_COMMITTEE_PARTICIPANTS: u64 = 1;

/// https://github.com/ethereum/consensus-specs/blob/dev/presets/mainnet/altair.yaml#L18
const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;

const SECONDS_PER_SLOT: u64 = 12;

const SLOTS_PER_EPOCH: u64 = 32;

// https://github.com/ethereum/consensus-specs/blob/dev/presets/mainnet/altair.yaml#L16
//const SYNC_COMMITTEE_SIZE: u64 = 512;

/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#misc
const UPDATE_TIMEOUT: u64 = SLOTS_PER_EPOCH * EPOCHS_PER_SYNC_COMMITTEE_PERIOD;

// Since each field in BeaconState has a known, and non-changing location in the merklized tree
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#constants
const FINALIZED_ROOT_INDEX: u64 = 105;

// Since each field in BeaconState has a known, and non-changing location in the merklized tree
// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#constants
const NEXT_SYNC_COMMITTEE_INDEX: u64 = 55;

/// DomainType('0x07000000')
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#domain-types
const DOMAIN_SYNC_COMMITTEE: [u8; 8] = [30, 37, 30, 30, 30, 30, 30, 30];

/// GENESIS_FORK_VERSION('0x00000000')
const GENESIS_FORK_VERSION: [u8; 8] = [30, 30, 30, 30, 30, 30, 30, 30];

type Epoch = u64;
type Slot = u64;
type Root = H256;
type Domain = H256;
type ValidatorIndex = u64;
type Version = Vec<u8>;

/// Beacon block header as it is stored in the runtime storage.
#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode, TreeHash,
)]
// https://yeeth.github.io/BeaconChain.swift/Structs/BeaconBlockHeader.html#/s:11BeaconChain0A11BlockHeaderV9signature10Foundation4DataVvp
// https://benjaminion.xyz/eth2-annotated-spec/phase0/beacon-chain/#beaconblock
// https://github.com/ethereum/consensus-specs/blob/042ca57a617736e6bdd6f6dcdd6d32c247e5a67f/specs/phase0/beacon-chain.md#beaconblockheader
pub struct BeaconBlockHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	#[tree_hash]
	pub slot: Slot,
	// The index of the validator that proposed the block.
	#[tree_hash]
	pub proposer_index: ValidatorIndex,
	// The block root of the parent block, forming a block chain.
	#[tree_hash]
	pub parent_root: Root,
	// The hash root of the post state of running the state transition through this block.
	#[tree_hash]
	pub state_root: Root,
	// The hash root of the Eth1 block
	#[tree_hash]
	pub body_root: Root,
}

// Only used for testing purposes
#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode, TreeHash,
)]
pub struct Checkpoint {
	#[tree_hash]
	pub epoch: u64,
	#[tree_hash]
	pub root: [u8; 32],
}

#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode, TreeHash,
)]
pub struct BeaconBlockHeader2 {
	// The slot for which this block is created. Must be greater than the slot of the block defined by parentRoot.
	#[tree_hash]
	pub slot: u64,
	// The index of the validator that proposed the block.
	#[tree_hash]
	pub proposer_index: u64,
	// The block root of the parent block, forming a block chain.
	#[tree_hash]
	pub parent_root: [u8; 32],
	// The hash root of the post state of running the state transition through this block.
	#[tree_hash]
	pub state_root: [u8; 32],
	// The hash root of the Eth1 block
	#[tree_hash]
	pub body_root: [u8; 32],
}

/// Sync committee as it is stored in the runtime storage.
/// https://github.com/ethereum/consensus-specs/blob/02b32100ed26c3c7a4a44f41b932437859487fd2/specs/altair/beacon-chain.md#synccommittee
#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode,
)]
pub struct SyncCommittee {
	pub pubkeys: Vec<Vec<u8>>,
	pub aggregate_pubkey: Vec<u8>,
}

/// https://github.com/ethereum/consensus-specs/blob/02b32100ed26c3c7a4a44f41b932437859487fd2/specs/altair/beacon-chain.md#syncaggregate
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
}

#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode,
)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: Version,
	pub genesis_validators_root: Vec<u8>,
}

#[derive(
	Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, SSZDecode, SSZEncode,
)]
pub struct SigningData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub object_root: Root,
	pub domain: Domain,
}

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
	pub pubfork_version: Option<Version>,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
use tree_hash::{merkle_root, merkleize_standard, merkleize_padded};

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
		InsufficientSyncCommitteeParticipants,
		InvalidSyncCommiteeSignature,
		InvalidMerkleProof,
		InvalidSignature,
		InvalidSignaturePoint,
		InvalidAggregatePublicKeys,
		SignatureVerificationFailed,
		NoBranchExpected,
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
		StorageValue<_, Option<LightClientUpdate>, ValueQuery>; // TODO: Maybe I should use OptionQuery here instead of Option?

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
		pub fn light_client_update(
			origin: OriginFor<T>,
			update: LightClientUpdate,
			current_slot: u64,
			genesis_validators_root: Root,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"Received update {:?}. Starting validation",
				update
			);

			Self::process_light_client_update(update, current_slot, genesis_validators_root)
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_light_client_update(
			update: LightClientUpdate,
			current_slot: u64,
			genesis_validators_root: Root,
		) -> DispatchResult {
			Self::validate_light_client_update(
				update.clone(),
				current_slot,
				genesis_validators_root,
			)?;

			let sync_committee_bits = update.clone().sync_aggregate.sync_committee_bits;

			let sync_committee_sum = Self::get_sync_committee_sum(sync_committee_bits.clone());

			// Update the best update in case we have to force-update to it if the timeout elapses
			if <BestValidUpdate<T>>::get().is_none()
				|| (sync_committee_sum
					> Self::get_sync_committee_sum(
						<BestValidUpdate<T>>::get().unwrap().sync_aggregate.sync_committee_bits,
					)) {
				// unwrap should be safe here because of short circuiting
				<BestValidUpdate<T>>::put(Some(update.clone()));
			}

			// Track the maximum number of active participants in the committee signatures
			<CurrentMaxActiveParticipants<T>>::put(cmp::max(
				<CurrentMaxActiveParticipants<T>>::get(),
				sync_committee_sum,
			));

			// Update the optimistic header
			if sync_committee_sum
				> Self::get_safety_threshold(
					<PreviousMaxActiveParticipants<T>>::get(),
					<CurrentMaxActiveParticipants<T>>::get(),
				) && update.clone().attested_header.slot > <OptimisticHeader<T>>::get().slot
			{
				<OptimisticHeader<T>>::put(update.clone().attested_header);
			}

			// Update finalized header if sync commitee votes were 2/3 or more
			if (sync_committee_sum * 3 >= sync_committee_bits.clone().len() as u64 * 2)
				&& update.clone().finalized_header.is_none()
			{
				// Normal update through 2/3 threshold
				Self::apply_light_client_update(update);
				<BestValidUpdate<T>>::kill();
			}

			Ok(())
		}

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

			let floor_log_2_finalized_root_index = Self::floorlog2(FINALIZED_ROOT_INDEX);

			// Verify that the `finalized_header`, if present, actually is the finalized header saved in the
			// state of the `attested header`.
			if update.finalized_header.is_none() {
				ensure!(update.finality_branch.len() == 0, Error::<T>::NoBranchExpected); 
			} else {
				ensure!(
					Self::is_valid_merkle_branch(
						Self::hash_tree_root(update.finalized_header.unwrap()),
						update.finality_branch,
						floor_log_2_finalized_root_index,
						Self::get_subtree_index(FINALIZED_ROOT_INDEX),
						update.attested_header.state_root
					),
					Error::<T>::InvalidMerkleProof
				);
			}

			let mut sync_committee: SyncCommittee;

			// Verify update next sync committee if the update period incremented
			if update_period == finalized_period {
				sync_committee = <CurrentSyncCommittee<T>>::get();
				ensure!(update.next_sync_committee_branch.len() == 0, Error::<T>::NoBranchExpected); 
			} else {
				sync_committee = <NextSyncCommittee<T>>::get();

				ensure!(
					Self::is_valid_merkle_branch(
						Self::hash_tree_root(update.next_sync_committee),
						update.next_sync_committee_branch,
						Self::floorlog2(NEXT_SYNC_COMMITTEE_INDEX),
						Self::get_subtree_index(NEXT_SYNC_COMMITTEE_INDEX),
						active_header.state_root
					),
					Error::<T>::InvalidMerkleProof
				);
			}
			let sync_aggregate = update.sync_aggregate.clone();

			ensure!(
				Self::get_sync_committee_sum(sync_aggregate.clone().sync_committee_bits)
					>= MIN_SYNC_COMMITTEE_PARTICIPANTS as u64,
				Error::<T>::InsufficientSyncCommitteeParticipants
			);

			// Verify sync committee aggregate signature
			let mut participant_pubkeys: Vec<Vec<u8>> = Vec::new();

			for (bit, pubkey) in sync_aggregate
				.clone()
				.sync_committee_bits
				.iter()
				.zip(sync_committee.pubkeys.iter_mut())
			{
				if *bit == 1 as u8 {
					let pubk = pubkey.clone();
					participant_pubkeys.push(pubk);
				}
			}

			let domain = Self::compute_domain(
				DOMAIN_SYNC_COMMITTEE.to_vec(),
				update.pubfork_version,
				genesis_validators_root,
			);

			let signing_root = Self::compute_signing_root(update.attested_header, domain);

			Self::bls_fast_aggregate_verify(
				participant_pubkeys,
				signing_root,
				sync_aggregate.sync_committee_signature
			)?;

			Ok(())
		}

		/// Is triggered every time the current slot increments.
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

		/// Returns the epooch at the specified slot.
		fn compute_epoch_at_slot(slot: Slot) -> Epoch {
			slot / SLOTS_PER_EPOCH
		}

		/// Return the sync committee period at epoch.
		fn compute_sync_committee_period(epoch: Epoch) -> u64 {
			epoch / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
		}

		pub fn is_valid_merkle_branch_wrapper<Z: SSZEncode>(
			leaf: Z,
			branch: Vec<H256>,
			root_index: u64,
			root: Root,
		) -> bool {
			Self::is_valid_merkle_branch(
				Self::hash_tree_root(leaf),
				branch,
				Self::floorlog2(root_index),
				Self::get_subtree_index(root_index),
				root,
			)
		}

		pub fn is_valid_merkle_branch(
			leaf: H256,
			branch: Vec<H256>,
			depth: u64,
			index: u64,
			root: Root,
		) -> bool {
			let mut value = leaf;
			for i in 0..depth {
				let base: u32 = 2;
				if (index / (base.pow(i as u32) as u64) % 2) == 0 {
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

		//** Helper functions **//
		// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/sync-protocol.md#helper-functions
		fn get_subtree_index(generalized_index: u64) -> u64 {
			let base: u32 = 2;

			let floorlog2_finalized_root_index = Self::floorlog2(FINALIZED_ROOT_INDEX);

			generalized_index % base.pow(floorlog2_finalized_root_index as u32) as u64
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

		fn get_sync_committee_sum(sync_committee_bits: Vec<u8>) -> u64 {
			sync_committee_bits.iter().fold(0, |acc: u64, x| acc + *x as u64)
		}

		/// Return the domain for the domain_type and fork_version.
		fn compute_domain(
			domain_type: Vec<u8>,
			fork_version: Option<Version>,
			genesis_validators_root: Root,
		) -> Domain {
			let unwrapped_fork_version: Version;
			if fork_version.is_none() {
				unwrapped_fork_version = GENESIS_FORK_VERSION.to_vec();
			} else {
				unwrapped_fork_version = fork_version.unwrap();
			}
			// TODO this may not be needed because we pass genesis_validators_root from relayer.
			//if genesis_validators_root is None:
			//	genesis_validators_root = Root()  # all bytes zero by default

			let fork_data_root =
				Self::compute_fork_data_root(unwrapped_fork_version, genesis_validators_root);

			let mut domain = [0u8; 32];

			domain[0..4].copy_from_slice(&(domain_type));
			domain[4..32].copy_from_slice(&(genesis_validators_root.0));

			domain.into()
		}

		fn compute_fork_data_root(current_version: Version, genesis_validators_root: Root) -> Root {
			Self::hash_tree_root(ForkData {
				current_version,
				genesis_validators_root: genesis_validators_root.as_bytes().to_vec(), // TODO maybe change type to Vec<u8> from the start
			})
		}

		fn compute_signing_root(ssz_object: BeaconBlockHeader, domain: Domain) -> Root {
			Self::hash_tree_root(SigningData {
				object_root: Self::hash_tree_root(ssz_object),
				domain,
			})
		}

		pub fn bls_fast_aggregate_verify(
			pubkeys: Vec<Vec<u8>>,
			message: H256,
			signature: Vec<u8>,
		) -> DispatchResult {
			let sig = Signature::from_bytes(&signature[..]);

			if let Err(e) = sig {
				return Err(Error::<T>::InvalidSignature.into());
			}

			let agg_sig = AggregateSignature::from_signature(&sig.unwrap());

			let public_keys_res: Result<Vec<PublicKey>, _> = pubkeys
				.iter()
				.map(|bytes| {
					PublicKey::from_bytes(&bytes)
				})
				.collect();

			if let Err(e) = public_keys_res {
				match e {
    				AmclError::InvalidPoint => return Err(Error::<T>::InvalidSignaturePoint.into()),
					_ => return Err(Error::<T>::InvalidSignature.into()),
				};
			}

			let agg_pub_key_res = AggregatePublicKey::into_aggregate(&public_keys_res.unwrap()); 

			if let Err(e) = agg_pub_key_res {
				return Err(Error::<T>::InvalidAggregatePublicKeys.into());
			}

			ensure!(agg_sig.fast_aggregate_verify_pre_aggregated(&message.as_bytes(), &agg_pub_key_res.unwrap()), Error::<T>::SignatureVerificationFailed);

			Ok(())
		}

		pub fn floorlog2(num: u64) -> u64 {
			(num as f64).log2().floor() as u64
		}

		pub fn hash_tree_root<Z: SSZEncode>(object: Z) -> Root {
			let ssz_bytes = Self::ssz_encode(object);

			merkle_root(&ssz_bytes, 0)
		}

		pub fn ssz_encode<Z: SSZEncode>(object: Z) -> Vec<u8> {
			object.as_ssz_bytes()
		}
	}
}
