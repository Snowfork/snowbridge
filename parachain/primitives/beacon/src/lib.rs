#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use snowbridge_ethereum::mpt;
use sp_core::{H160, H256, U256};
use sp_io::hashing::keccak_256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use core::fmt::Formatter;
#[cfg(feature = "std")]
use serde::{de::Error, de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "std")]
use sp_std::fmt::Result as StdResult;

pub type Root = H256;
pub type Domain = H256;
pub type ValidatorIndex = u64;
pub type ProofBranch = Vec<H256>;
pub type ForkVersion = [u8; 4];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey(pub [u8; 48]);

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; 48])
	}
}

#[cfg(feature = "std")]
impl Serialize for PublicKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(&self.0)
	}
}

struct PublicKeyVisitor;

#[cfg(feature = "std")]
impl<'de> Visitor<'de> for PublicKeyVisitor {
	type Value = PublicKey;

	fn expecting(&self, formatter: &mut Formatter) -> StdResult {
		formatter.write_str("a hex string")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: Error,
	{
		let str_without_0x = match v.strip_prefix("0x") {
			Some(val) => val,
			None => v,
		};

		let hex_bytes = match hex::decode(str_without_0x) {
			Ok(bytes) => bytes,
			Err(e) => return Err(Error::custom(e.to_string())),
		};
		if hex_bytes.len() != 48 {
			return Err(Error::custom("publickey expected to be 48 characters"))
		}

		let mut data = [0u8; 48];
		data[0..48].copy_from_slice(&hex_bytes);
		Ok(PublicKey(data))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for PublicKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(PublicKeyVisitor)
	}
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialSync {
	pub header: BeaconHeader,
	pub current_sync_committee: SyncCommittee,
	pub current_sync_committee_branch: ProofBranch,
	pub validators_root: Root,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncCommitteePeriodUpdate {
	pub attested_header: BeaconHeader,
	pub next_sync_committee: SyncCommittee,
	pub next_sync_committee_branch: ProofBranch,
	pub finalized_header: BeaconHeader,
	pub finality_branch: ProofBranch,
	pub sync_aggregate: SyncAggregate,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_fork_version"))]
	pub fork_version: ForkVersion,
	pub sync_committee_period: u64,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct FinalizedHeaderUpdate {
	pub attested_header: BeaconHeader,
	pub finalized_header: BeaconHeader,
	pub finality_branch: ProofBranch,
	pub sync_aggregate: SyncAggregate,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_fork_version"))]
	pub fork_version: ForkVersion,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BlockUpdate {
	pub block: BeaconBlock,
	//  // Only used for debugging purposes, to compare the hash tree
	// root of the block body to the body hash retrieved from the API.
	// Can be removed later.
	pub block_body_root: H256,
	pub sync_aggregate: SyncAggregate,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_fork_version"))]
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
pub struct ExecutionHeader {
	pub parent_hash: H256,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: Vec<u8>,
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
}

/// Sync committee as it is stored in the runtime storage.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SyncCommittee {
	pub pubkeys: Vec<PublicKey>,
	pub aggregate_pubkey: PublicKey,
}

/// Beacon block header as it is stored in the runtime storage. The block root is the
/// Merklization of a BeaconHeader.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BeaconHeader {
	// The slot for which this block is created. Must be greater than the slot of the block defined
	// by parentRoot.
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

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DepositData {
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub pubkey: Vec<u8>,
	pub withdrawal_credentials: H256,
	pub amount: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Deposit {
	pub proof: Vec<H256>,
	pub data: DepositData,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Checkpoint {
	pub epoch: u64,
	pub root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AttestationData {
	pub slot: u64,
	pub index: u64,
	pub beacon_block_root: H256,
	pub source: Checkpoint,
	pub target: Checkpoint,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IndexedAttestation {
	pub attesting_indices: Vec<u64>,
	pub data: AttestationData,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SignedHeader {
	pub message: crate::BeaconHeader,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ProposerSlashing {
	pub signed_header_1: SignedHeader,
	pub signed_header_2: SignedHeader,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AttesterSlashing {
	pub attestation_1: IndexedAttestation,
	pub attestation_2: IndexedAttestation,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Attestation {
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub aggregation_bits: Vec<u8>,
	pub data: AttestationData,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VoluntaryExit {
	pub epoch: u64,
	pub validator_index: u64,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Eth1Data {
	pub deposit_root: H256,
	pub deposit_count: u64,
	pub block_hash: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SyncAggregate {
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub sync_committee_bits: Vec<u8>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub sync_committee_signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ExecutionPayload {
	pub parent_hash: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub fee_recipient: Vec<u8>,
	pub state_root: H256,
	pub receipts_root: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub extra_data: Vec<u8>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_int_to_u256"))]
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Body {
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub randao_reveal: Vec<u8>,
	pub eth1_data: Eth1Data,
	pub graffiti: H256,
	pub proposer_slashings: Vec<ProposerSlashing>,
	pub attester_slashings: Vec<AttesterSlashing>,
	pub attestations: Vec<Attestation>,
	pub deposits: Vec<Deposit>,
	pub voluntary_exits: Vec<VoluntaryExit>,
	pub sync_aggregate: SyncAggregate,
	pub execution_payload: ExecutionPayload,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct BeaconBlock {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: H256,
	pub state_root: H256,
	pub body: Body,
}

impl ExecutionHeader {
	// Copied from ethereum_snowbridge::header
	pub fn check_receipt_proof(
		&self,
		proof: &[Vec<u8>],
	) -> Option<Result<snowbridge_ethereum::Receipt, rlp::DecoderError>> {
		match self.apply_merkle_proof(proof) {
			Some((root, data)) if root == self.receipts_root => Some(rlp::decode(&data)),
			Some((_, _)) => None,
			None => None,
		}
	}

	// Copied from ethereum_snowbridge::header
	pub fn apply_merkle_proof(&self, proof: &[Vec<u8>]) -> Option<(H256, Vec<u8>)> {
		let mut iter = proof.into_iter().rev();
		let first_bytes = match iter.next() {
			Some(b) => b,
			None => return None,
		};
		let item_to_prove: mpt::ShortNode = rlp::decode(first_bytes).ok()?;

		let final_hash: Option<[u8; 32]> =
			iter.fold(Some(keccak_256(first_bytes)), |maybe_hash, bytes| {
				let expected_hash = maybe_hash?;
				let node: Box<dyn mpt::Node> = bytes.as_slice().try_into().ok()?;
				if (*node).contains_hash(expected_hash.into()) {
					return Some(keccak_256(bytes))
				}
				None
			});

		final_hash.map(|hash| (hash.into(), item_to_prove.value))
	}
}

#[cfg(feature = "std")]
fn from_hex_to_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?; 

	let str_without_0x = match s.strip_prefix("0x") {
		Some(val) => val,
		None => &s,
	};

	let hex_bytes = match hex::decode(str_without_0x) {
		Ok(bytes) => bytes,
		Err(e) => return Err(Error::custom(e.to_string())),
	};

	Ok(hex_bytes)
}

#[cfg(feature = "std")]
fn from_hex_to_fork_version<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?; 

	let str_without_0x = match s.strip_prefix("0x") {
		Some(val) => val,
		None => &s,
	};

	let hex_bytes = match hex::decode(str_without_0x) {
		Ok(bytes) => bytes,
		Err(e) => return Err(Error::custom(e.to_string())),
	};

	if hex_bytes.len() != 4 {
		return Err(Error::custom("fork version expected to be 4 characters"))
	}

	let mut data = [0u8; 4];
	data[0..4].copy_from_slice(&hex_bytes);

	Ok(data)
}

#[cfg(feature = "std")]
fn from_int_to_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: Deserializer<'de>,
{
	let number = u128::deserialize(deserializer)?; 

	Ok(U256::from(number))
}