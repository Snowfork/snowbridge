//! # Ethereum Beacon Client
#![cfg_attr(not(feature = "std"), no_std)]

mod merklization;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, log, transactional};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_io::hashing::sha2_256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

type Root = H256;
type Domain = H256;
type ValidatorIndex = u64;
type ProofBranch = Vec<H256>;
type ForkVersion = [u8; 4];

const SLOTS_PER_EPOCH: u64 = 32;

const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;

const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 22;
const CURRENT_SYNC_COMMITTEE_DEPTH: u64 = 5;

const NEXT_SYNC_COMMITTEE_DEPTH: u64 = 5;
const NEXT_SYNC_COMMITTEE_INDEX: u64 = 23;

const FINALIZED_ROOT_DEPTH: u64 = 6;
const FINALIZED_ROOT_INDEX: u64 = 41;

/// GENESIS_FORK_VERSION('0x00000000')
const GENESIS_FORK_VERSION: ForkVersion = [30, 30, 30, 30];

/// DomainType('0x07000000')
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#domain-types
const DOMAIN_SYNC_COMMITTEE: [u8; 4] = [7, 0, 0, 0];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey([u8; 48]);

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; 48])
	}
}

/// Beacon block header as it is stored in the runtime storage. The block root is the
/// Merklization of a BeaconHeader.
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
	// The hash root of the beacon block body
	pub body_root: Root,
}

/// Sync committee as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncCommittee {
	pub pubkeys: Vec<PublicKey>,
	pub aggregate_pubkey: PublicKey,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct InitialSync {
	pub header: BeaconBlockHeader,
	pub current_sync_committee: SyncCommittee,
	pub current_sync_committee_branch: ProofBranch,
	pub validators_root: Root,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncCommitteePeriodUpdate {
	pub attested_header: BeaconBlockHeader,
	pub next_sync_committee: SyncCommittee,
	pub next_sync_committee_branch: ProofBranch,
	pub finalized_header: BeaconBlockHeader,
	pub finality_branch: ProofBranch,
	pub sync_aggregate: SyncAggregate,
	pub fork_version: ForkVersion,
	pub sync_committee_period: u64,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct FinalizedHeaderUpdate {
	pub attested_header: BeaconBlockHeader,
	pub finalized_header: BeaconBlockHeader,
	pub finality_branch: ProofBranch,
	pub sync_aggregate: SyncAggregate,
	pub fork_version: ForkVersion,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SigningData {
	pub object_root: Root,
	pub domain: Domain,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Genesis {
	pub validators_root: Root,
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use milagro_bls::{AggregatePublicKey, AggregateSignature, AmclError, Signature};
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
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
		SyncCommitteeMissing,
		Unknown,
		SyncCommitteeParticipantsNotSupermajority,
		InvalidSyncCommiteeSignature,
		InvalidHeaderMerkleProof,
		InvalidSyncCommitteeMerkleProof,
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
	pub(super) type FinalizedHeaders<T: Config> =
		StorageMap<_, Identity, H256, BeaconBlockHeader, OptionQuery>;

	#[pallet::storage]
	pub(super) type FinalizedHeadersBySlot<T: Config> =
		StorageMap<_, Identity, u64, H256, OptionQuery>;

	/// Current sync committee corresponding to the active header.
	/// TODO  prune older sync committees than xxx
	#[pallet::storage]
	pub(super) type SyncCommittees<T: Config> =
		StorageMap<_, Identity, u64, SyncCommittee, ValueQuery>;

	#[pallet::storage]
	pub(super) type ChainGenesis<T: Config> = StorageValue<_, Genesis, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {}

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
			initial_sync: InitialSync,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Received initial sync, starting processing.",
			);

			if let Err(err) = Self::process_initial_sync(initial_sync) {
				log::error!(
					target: "ethereum-beacon-client",
					"Initial sync failed with error {:?}",
					err
				);
				return Err(err);
			}

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Initial sync processing succeeded.",
			);

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn sync_committee_period_update(
			origin: OriginFor<T>,
			sync_committee_period_update: SyncCommitteePeriodUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			let sync_committee_period = sync_committee_period_update.sync_committee_period;
			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Received sync committee update for period {}. Applying update",
				sync_committee_period
			);

			if let Err(err) = Self::process_sync_committee_period_update(sync_committee_period_update) {
				log::error!(
					target: "ethereum-beacon-client",
					"Sync committee period update failed with error {:?}",
					err
				);
				return Err(err);
			}

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Sync committee period update for period {} succeeded.",
				sync_committee_period
			);

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn import_finalized_header(
			origin: OriginFor<T>,
			finalized_header_update: FinalizedHeaderUpdate,
		) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			let slot = finalized_header_update.finalized_header.slot;

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Received finalized header update for slot {}, processing and importing finalized header.",
				slot
			);

			if let Err(err) = Self::process_finalized_header(finalized_header_update) {
				log::error!(
					target: "ethereum-beacon-client",
					"Finalized header update failed with error {:?}",
					err
				);
				return Err(err);
			}

			log::trace!(
				target: "ethereum-beacon-client",
				"💫 Finalized header processing and importing at slot {} succeeded.",
				slot
			);

			Ok(())
		}

		#[pallet::weight(1_000_000)]
		#[transactional]
		pub fn verify_eth1_receipt_inclusion(
			origin: OriginFor<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			log::trace!(
				target: "ethereum-beacon-light-client",
				"💫 Received transaction to be validated.",
			);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn process_initial_sync(initial_sync: InitialSync) -> DispatchResult {
			Self::verify_sync_committee(
				initial_sync.current_sync_committee.clone(),
				initial_sync.current_sync_committee_branch,
				initial_sync.header.state_root,
				CURRENT_SYNC_COMMITTEE_DEPTH,
				CURRENT_SYNC_COMMITTEE_INDEX,
			)?;

			let period = Self::compute_current_sync_period(initial_sync.header.slot);
			Self::store_sync_committee(period, initial_sync.current_sync_committee);

			let block_root: H256 = merklization::hash_tree_root_beacon_header(initial_sync.header.clone())
				.map_err(|_| DispatchError::Other("Header hash tree root failed"))?.into();
			Self::store_header(block_root, initial_sync.header);

			Self::store_genesis(Genesis { validators_root: initial_sync.validators_root });

			Ok(())
		}

		fn process_sync_committee_period_update(
			update: SyncCommitteePeriodUpdate,
		) -> DispatchResult {
			let sync_committee_bits = Self::convert_to_binary(update.sync_aggregate.sync_committee_bits.clone());
			Self::sync_committee_participation_is_supermajority(sync_committee_bits.clone())?;
			Self::verify_sync_committee(
				update.next_sync_committee.clone(),
				update.next_sync_committee_branch,
				update.finalized_header.state_root,
				NEXT_SYNC_COMMITTEE_DEPTH,
				NEXT_SYNC_COMMITTEE_INDEX,
			)?;

			let block_root: H256 = merklization::hash_tree_root_beacon_header(update.finalized_header.clone())
				.map_err(|_| DispatchError::Other("Header hash tree root failed"))?.into();
			Self::verify_header(
				block_root,
				update.finality_branch,
				update.attested_header.state_root,
				FINALIZED_ROOT_DEPTH,
				FINALIZED_ROOT_INDEX,
			)?;

			let current_period = Self::compute_current_sync_period(update.attested_header.slot);
			Self::store_sync_committee(current_period + 1, update.next_sync_committee);

			let current_sync_committee = <SyncCommittees<T>>::get(current_period);
			let genesis = <ChainGenesis<T>>::get();

			Self::verify_signed_header(
				sync_committee_bits,
				update.sync_aggregate.sync_committee_signature,
				current_sync_committee.pubkeys,
				update.fork_version,
				update.attested_header,
				genesis.validators_root,
			)?;

			Self::store_header(block_root, update.finalized_header);

			Ok(())
		}

		fn process_finalized_header(update: FinalizedHeaderUpdate) -> DispatchResult {
			let sync_committee_bits = Self::convert_to_binary(update.sync_aggregate.sync_committee_bits.clone());
			Self::sync_committee_participation_is_supermajority(sync_committee_bits.clone())?;

			let block_root: H256 = merklization::hash_tree_root_beacon_header(update.finalized_header.clone())
				.map_err(|_| DispatchError::Other("Header hash tree root failed"))?.into();
			Self::verify_header(
				block_root,
				update.finality_branch,
				update.attested_header.state_root,
				FINALIZED_ROOT_DEPTH,
				FINALIZED_ROOT_INDEX,
			)?;

			let current_period = Self::compute_current_sync_period(update.attested_header.slot);
			let sync_committee = <SyncCommittees<T>>::get(current_period);
			if (SyncCommittee { pubkeys: vec![], aggregate_pubkey: PublicKey([0; 48]) }) == sync_committee {
				return Err(Error::<T>::SyncCommitteeMissing.into());
			}
			let genesis = <ChainGenesis<T>>::get();
			Self::verify_signed_header(
				sync_committee_bits,
				update.sync_aggregate.sync_committee_signature,
				sync_committee.pubkeys,
				update.fork_version,
				update.attested_header,
				genesis.validators_root,
			)?;

			Self::store_header(block_root, update.finalized_header);

			Ok(())
		}

		pub(super) fn verify_signed_header(
			sync_committee_bits: Vec<u8>,
			sync_committee_signature: Vec<u8>,
			sync_committee_pubkeys: Vec<PublicKey>,
			fork_version: ForkVersion,
			header: BeaconBlockHeader,
			validators_root: H256,
		) -> DispatchResult {
			let mut participant_pubkeys: Vec<PublicKey> = Vec::new();
			// Gathers all the pubkeys of the sync committee members that participated in siging the header.
			for (bit, pubkey) in sync_committee_bits.iter().zip(sync_committee_pubkeys.iter()) {
				if *bit == 1 as u8 {
					let pubk = pubkey.clone();
					participant_pubkeys.push(pubk);
				}
			}

			let domain_type = DOMAIN_SYNC_COMMITTEE.to_vec();
			// Domains are used for for seeds, for signatures, and for selecting aggregators.
			let domain = Self::compute_domain(domain_type, Some(fork_version), validators_root)?;
			// Hash tree root of SigningData - object root + domain
			let signing_root = Self::compute_signing_root(header, domain)?;

			// Verify sync committee aggregate signature.
			Self::bls_fast_aggregate_verify(
				participant_pubkeys,
				signing_root,
				sync_committee_signature,
			)?;

			Ok(())
		}

		pub(super) fn bls_fast_aggregate_verify(
			pubkeys: Vec<PublicKey>,
			message: H256,
			signature: Vec<u8>,
		) -> DispatchResult {
			let sig = Signature::from_bytes(&signature[..]);
			if let Err(_e) = sig {
				return Err(Error::<T>::InvalidSignature.into());
			}

			let agg_sig = AggregateSignature::from_signature(&sig.unwrap());

			let public_keys_res: Result<Vec<milagro_bls::PublicKey>, _> =
				pubkeys.iter().map(|bytes| milagro_bls::PublicKey::from_bytes_unchecked(&bytes.0)).collect();
			if let Err(e) = public_keys_res {
				match e {
					AmclError::InvalidPoint => return Err(Error::<T>::InvalidSignaturePoint.into()),
					_ => return Err(Error::<T>::InvalidSignature.into()),
				};
			}

			let agg_pub_key_res = AggregatePublicKey::into_aggregate(&public_keys_res.unwrap());
			if let Err(_e) = agg_pub_key_res {
				return Err(Error::<T>::InvalidAggregatePublicKeys.into());
			}

			ensure!(
				agg_sig.fast_aggregate_verify_pre_aggregated(
					&message.as_bytes(),
					&agg_pub_key_res.unwrap()
				),
				Error::<T>::SignatureVerificationFailed
			);

			Ok(())
		}

		pub(super) fn compute_signing_root(
			beacon_header: BeaconBlockHeader,
			domain: Domain,
		) -> Result<Root, DispatchError> {
			let beacon_header_root = merklization::hash_tree_root_beacon_header(beacon_header)
				.map_err(|_| DispatchError::Other("Beacon header hash tree root failed"))?;

			let hash_root = merklization::hash_tree_root_signing_data(SigningData {
				object_root: beacon_header_root.into(),
				domain,
			})
			.map_err(|_| DispatchError::Other("Signing root hash tree root failed"))?;

			Ok(hash_root.into())
		}

		fn verify_sync_committee(
			sync_committee: SyncCommittee,
			sync_committee_branch: ProofBranch,
			header_state_root: H256,
			depth: u64,
			index: u64,
		) -> DispatchResult {
			let sync_committee_root =
				merklization::hash_tree_root_sync_committee(sync_committee)
					.map_err(|_| DispatchError::Other("Sync committee hash tree root failed"))?;

			ensure!(
				Self::is_valid_merkle_branch(
					sync_committee_root.into(),
					sync_committee_branch,
					depth,
					index,
					header_state_root
				),
				Error::<T>::InvalidSyncCommitteeMerkleProof
			);

			Ok(())
		}

		fn verify_header(
			block_root: H256,
			proof_branch: ProofBranch,
			attested_header_state_root: H256,
			depth: u64,
			index: u64,
		) -> DispatchResult {
			ensure!(
				Self::is_valid_merkle_branch(
					block_root,
					proof_branch,
					depth,
					index,
					attested_header_state_root
				),
				Error::<T>::InvalidHeaderMerkleProof
			);

			Ok(())
		}

		fn store_sync_committee(period: u64, sync_committee: SyncCommittee) {
			<SyncCommittees<T>>::insert(period, sync_committee);
		}

		fn store_header(block_root: H256, header: BeaconBlockHeader) {
			<FinalizedHeaders<T>>::insert(block_root, header.clone());

			<FinalizedHeadersBySlot<T>>::insert(header.slot, block_root);
		}

		fn store_genesis(genesis: Genesis) {
			<ChainGenesis<T>>::put(genesis);
		}

		/// Sums the bit vector of sync committee particpation.
		///
		/// # Examples
		///
		/// let sync_committee_bits = vec![0, 1, 0, 1, 1, 1];
		/// ensure!(get_sync_committee_sum(sync_committee_bits), 4);
		pub(super) fn get_sync_committee_sum(sync_committee_bits: Vec<u8>) -> u64 {
			sync_committee_bits.iter().fold(0, |acc: u64, x| acc + *x as u64)
		}

		pub(super) fn compute_current_sync_period(slot: u64) -> u64 {
			slot / SLOTS_PER_EPOCH / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
		}

		/// Return the domain for the domain_type and fork_version.
		pub(super) fn compute_domain(
			domain_type: Vec<u8>,
			fork_version: Option<ForkVersion>,
			genesis_validators_root: Root,
		) -> Result<Domain, DispatchError> {
			let unwrapped_fork_version: ForkVersion;
			if fork_version.is_none() {
				unwrapped_fork_version = GENESIS_FORK_VERSION;
			} else {
				unwrapped_fork_version = fork_version.unwrap();
			}

			let fork_data_root =
				Self::compute_fork_data_root(unwrapped_fork_version, genesis_validators_root)?;

			let mut domain = [0u8; 32];
			domain[0..4].copy_from_slice(&(domain_type));
			domain[4..32].copy_from_slice(&(fork_data_root.0[..28]));

			Ok(domain.into())
		}

		fn compute_fork_data_root(
			current_version: ForkVersion,
			genesis_validators_root: Root,
		) -> Result<Root, DispatchError> {
			let hash_root = merklization::hash_tree_root_fork_data(ForkData {
				current_version,
				genesis_validators_root: genesis_validators_root.into(),
			})
			.map_err(|_| DispatchError::Other("Fork data hash tree root failed"))?;

			Ok(hash_root.into())
		}

		pub(super) fn is_valid_merkle_branch(
			leaf: H256,
			branch: Vec<H256>,
			depth: u64,
			index: u64,
			root: Root,
		) -> bool {
			if branch.len() != depth as usize {
				log::error!(target: "ethereum-beacon-client", "Merkle proof branch length doesn't match depth.");

				return false;
			}
			let mut value = leaf;
			if leaf.as_bytes().len() < 32 as usize {
				log::error!(target: "ethereum-beacon-client", "Merkle proof leaf not 32 bytes.");

				return false;
			}
			for i in 0..depth {
				if branch[i as usize].as_bytes().len() < 32 as usize {
					log::error!(target: "ethereum-beacon-client", "Merkle proof branch not 32 bytes.");

					return false;
				}
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

		pub(super) fn convert_to_binary(input: Vec<u8>) -> Vec<u8> {
			let mut result = Vec::new();

			for input_decimal in input.iter() {
				let mut tmp = Vec::new();
				let mut remaining = *input_decimal;

				while remaining > 0 {
					let remainder = remaining % 2;
					tmp.push(remainder);
					remaining = remaining / 2;
				}

				// pad binary with 0s if length is less than 7
				if tmp.len() < 8 {
					for _i in tmp.len()..8 {
						tmp.push(0)
					}
				}

				result.append(&mut tmp);
			}

			result
		}

		pub(super) fn sync_committee_participation_is_supermajority(sync_committee_bits: Vec<u8>) -> DispatchResult {
			let sync_committee_sum = Self::get_sync_committee_sum(sync_committee_bits.clone());
			ensure!(
				(sync_committee_sum * 3 >= sync_committee_bits.clone().len() as u64 * 2),
				Error::<T>::SyncCommitteeParticipantsNotSupermajority
			);

			Ok(())
		}
	}
}
