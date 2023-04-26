#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;
pub mod merkleization;
pub mod ssz;

#[cfg(feature = "std")]
mod serde_utils;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use snowbridge_ethereum::mpt;
use sp_core::{H160, H256, U256};
use sp_io::hashing::keccak_256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use core::fmt::Formatter;
#[cfg(feature = "std")]
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "std")]
use sp_std::fmt::Result as StdResult;

use config::{PUBKEY_SIZE, SIGNATURE_SIZE};

pub type Root = H256;
pub type Domain = H256;
pub type ValidatorIndex = u64;
pub type ForkVersion = [u8; 4];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ForkVersions {
	pub genesis: Fork,
	pub altair: Fork,
	pub bellatrix: Fork,
	pub capella: Fork,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Fork {
	pub version: [u8; 4],
	pub epoch: u64,
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey(pub [u8; PUBKEY_SIZE]);

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; PUBKEY_SIZE])
	}
}

impl From<[u8; PUBKEY_SIZE]> for PublicKey {
	fn from(v: [u8; PUBKEY_SIZE]) -> Self {
		Self(v)
	}
}

impl From<&[u8; PUBKEY_SIZE]> for PublicKey {
	fn from(v: &[u8; PUBKEY_SIZE]) -> Self {
		Self(*v)
	}
}

impl MaxEncodedLen for PublicKey {
	fn max_encoded_len() -> usize {
		PUBKEY_SIZE
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
		E: serde::de::Error,
	{
		let str_without_0x = match v.strip_prefix("0x") {
			Some(val) => val,
			None => v,
		};

		let hex_bytes = match hex::decode(str_without_0x) {
			Ok(bytes) => bytes,
			Err(e) => return Err(serde::de::Error::custom(e.to_string())),
		};
		if hex_bytes.len() != PUBKEY_SIZE {
			return Err(serde::de::Error::custom("publickey expected to be 48 characters"))
		}

		let mut data = [0u8; PUBKEY_SIZE];
		data[0..PUBKEY_SIZE].copy_from_slice(&hex_bytes);
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

#[cfg(feature = "std")]
impl Serialize for PublicKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_bytes(&self.0)
	}
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Signature(pub [u8; SIGNATURE_SIZE]);

impl Default for Signature {
	fn default() -> Self {
		Signature([0u8; SIGNATURE_SIZE])
	}
}

impl From<[u8; SIGNATURE_SIZE]> for Signature {
	fn from(v: [u8; SIGNATURE_SIZE]) -> Self {
		Self(v)
	}
}

impl From<&[u8; SIGNATURE_SIZE]> for Signature {
	fn from(v: &[u8; SIGNATURE_SIZE]) -> Self {
		Self(*v)
	}
}

struct SignatureVisitor;

#[cfg(feature = "std")]
impl<'de> Visitor<'de> for SignatureVisitor {
	type Value = Signature;

	fn expecting(&self, formatter: &mut Formatter) -> StdResult {
		formatter.write_str("a hex string")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: serde::de::Error,
	{
		let str_without_0x = match v.strip_prefix("0x") {
			Some(val) => val,
			None => v,
		};

		let hex_bytes = match hex::decode(str_without_0x) {
			Ok(bytes) => bytes,
			Err(e) => return Err(serde::de::Error::custom(e.to_string())),
		};
		if hex_bytes.len() != SIGNATURE_SIZE {
			return Err(serde::de::Error::custom("publickey expected to be 48 characters"))
		}

		let mut data = [0u8; SIGNATURE_SIZE];
		data[0..SIGNATURE_SIZE].copy_from_slice(&hex_bytes);
		Ok(Signature(data))
	}
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(SignatureVisitor)
	}
}

#[derive(Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ExecutionHeaderState {
	pub beacon_block_root: H256,
	pub beacon_slot: u64,
	pub block_hash: H256,
	pub block_number: u64,
}

#[derive(Default, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct FinalizedHeaderState {
	pub beacon_block_root: H256,
	pub beacon_slot: u64,
	pub import_time: u64,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct InitialSync<const SYNC_COMMITTEE_SIZE: usize> {
	pub header: BeaconHeader,
	pub current_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
	pub current_sync_committee_branch: Vec<H256>,
	pub validators_root: Root,
	pub import_time: u64,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Default for InitialSync<SYNC_COMMITTEE_SIZE> {
	fn default() -> Self {
		InitialSync {
			header: Default::default(),
			current_sync_committee: Default::default(),
			current_sync_committee_branch: Default::default(),
			validators_root: Default::default(),
			import_time: Default::default(),
		}
	}
}

#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct SyncCommitteePeriodUpdate<const SYNC_COMMITTEE_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub next_sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
	pub next_sync_committee_branch: Vec<H256>,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
	pub sync_committee_period: u64,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}

#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize,))]
#[codec(mel_bound())]
pub struct FinalizedHeaderUpdate<const SYNC_COMMITTEE_SIZE: usize> {
	pub attested_header: BeaconHeader,
	pub finalized_header: BeaconHeader,
	pub finality_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: Vec<H256>,
}

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct HeaderUpdate<const SYNC_COMMITTEE_SIZE: usize> {
	pub beacon_header: BeaconHeader,
	pub execution_header: ExecutionPayloadHeader,
	pub execution_branch: Vec<H256>,
	pub sync_aggregate: SyncAggregate<SYNC_COMMITTEE_SIZE>,
	pub signature_slot: u64,
	pub block_root_branch: Vec<H256>,
	pub block_root_branch_header_root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct SigningData {
	pub object_root: Root,
	pub domain: Domain,
}

#[derive(
	Default,
	Encode,
	Decode,
	CloneNoBound,
	PartialEqNoBound,
	RuntimeDebugNoBound,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct ExecutionHeader {
	pub parent_hash: H256,
	pub block_hash: H256,
	pub block_number: u64,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
}

#[derive(Debug, PartialEq)]
pub enum ConvertError {
	FromExecutionPayloadToHeaderError,
}

impl TryFrom<ExecutionPayloadHeader> for ExecutionHeader {
	type Error = ConvertError;

	fn try_from(execution_payload: ExecutionPayloadHeader) -> Result<Self, Self::Error> {
		Ok(ExecutionHeader {
			parent_hash: execution_payload.parent_hash,
			block_hash: execution_payload.block_hash,
			block_number: execution_payload.block_number,
			fee_recipient: H160::from(execution_payload.fee_recipient),
			state_root: execution_payload.state_root,
			receipts_root: execution_payload.receipts_root,
		})
	}
}

/// Sync committee as it is stored in the runtime storage.
#[derive(
	Encode, Decode, PartialEqNoBound, CloneNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct SyncCommittee<const SYNC_COMMITTEE_SIZE: usize> {
	#[cfg_attr(feature = "std", serde(with = "serde_utils::arrays"))]
	pub pubkeys: [PublicKey; SYNC_COMMITTEE_SIZE],
	pub aggregate_pubkey: PublicKey,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Default for SyncCommittee<SYNC_COMMITTEE_SIZE> {
	fn default() -> Self {
		SyncCommittee {
			pubkeys: [Default::default(); SYNC_COMMITTEE_SIZE],
			aggregate_pubkey: Default::default(),
		}
	}
}

/// Beacon block header as it is stored in the runtime storage. The block root is the
/// Merklization of a BeaconHeader.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize, SignatureSize))]
#[codec(mel_bound())]
pub struct SyncAggregate<const SYNC_COMMITTEE_SIZE: usize> {
	#[serde(with = "serde_utils::arrays")]
	pub sync_committee_bits: [u8; SYNC_COMMITTEE_SIZE],
	pub sync_committee_signature: Signature,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Default for SyncAggregate<SYNC_COMMITTEE_SIZE> {
	fn default() -> Self {
		SyncAggregate {
			sync_committee_bits: [0; SYNC_COMMITTEE_SIZE],
			sync_committee_signature: Default::default(),
		}
	}
}

#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[codec(mel_bound())]
pub struct ExecutionPayloadHeader {
	pub parent_hash: H256,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "serde_utils::from_hex_to_bytes"))]
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "serde_utils::from_hex_to_bytes"))]
	pub extra_data: Vec<u8>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "serde_utils::from_int_to_u256"))]
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
	pub withdrawals_root: H256,
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
