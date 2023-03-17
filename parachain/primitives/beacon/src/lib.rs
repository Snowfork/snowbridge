#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::Get, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
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

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey(pub [u8; 48]);

impl<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>> Default
	for InitialSync<SyncCommitteeSize, ProofSize>
{
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

impl<SyncCommitteeSize: Get<u32>> Default for SyncCommittee<SyncCommitteeSize> {
	fn default() -> Self {
		SyncCommittee { pubkeys: Default::default(), aggregate_pubkey: Default::default() }
	}
}

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; 48])
	}
}

impl MaxEncodedLen for PublicKey {
	fn max_encoded_len() -> usize {
		48
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

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize, ProofSize))]
#[codec(mel_bound())]
pub struct InitialSync<SyncCommitteeSize: Get<u32>, ProofSize: Get<u32>> {
	pub header: BeaconHeader,
	pub current_sync_committee: SyncCommittee<SyncCommitteeSize>,
	pub current_sync_committee_branch: BoundedVec<H256, ProofSize>,
	pub validators_root: Root,
	pub import_time: u64,
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SignatureSize, ProofSize, SyncCommitteeSize, SyncCommitteeSize))]
#[codec(mel_bound())]
pub struct SyncCommitteePeriodUpdate<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
> {
	pub attested_header: BeaconHeader,
	pub next_sync_committee: SyncCommittee<SyncCommitteeSize>,
	pub next_sync_committee_branch: BoundedVec<H256, ProofSize>,
	pub finalized_header: BeaconHeader,
	pub finality_branch: BoundedVec<H256, ProofSize>,
	pub sync_aggregate: SyncAggregate<SyncCommitteeSize, SignatureSize>,
	pub sync_committee_period: u64,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: BoundedVec<H256, ProofSize>,
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SignatureSize, ProofSize, SyncCommitteeSize,))]
#[codec(mel_bound())]
pub struct FinalizedHeaderUpdate<
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
> {
	pub attested_header: BeaconHeader,
	pub finalized_header: BeaconHeader,
	pub finality_branch: BoundedVec<H256, ProofSize>,
	pub sync_aggregate: SyncAggregate<SyncCommitteeSize, SignatureSize>,
	pub signature_slot: u64,
	pub block_roots_root: H256,
	pub block_roots_branch: BoundedVec<H256, ProofSize>,
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(
	FeeRecipientSize,
	LogsBloomSize,
	ExtraDataSize,
	DepositDataSize,
	PublicKeySize,
	SignatureSize,
	ProofSize,
	ProposerSlashingSize,
	AttesterSlashingSize,
	VoluntaryExitSize,
	AttestationSize,
	ValidatorCommitteeSize,
	SyncCommitteeSize
))]
#[codec(mel_bound())]
pub struct HeaderUpdate<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
	SignatureSize: Get<u32>,
	ProofSize: Get<u32>,
	SyncCommitteeSize: Get<u32>,
> {
	pub beacon_header: BeaconHeader,
	pub execution_header: VersionedExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>,
	pub execution_branch: BoundedVec<H256, ProofSize>,
	pub sync_aggregate: SyncAggregate<SyncCommitteeSize, SignatureSize>,
	pub signature_slot: u64,
	pub block_root_branch: BoundedVec<H256, ProofSize>,
	pub block_root_branch_header_root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ForkData {
	// 1 or 0 bit, indicates whether a sync committee participated in a vote
	pub current_version: [u8; 4],
	pub genesis_validators_root: [u8; 32],
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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
#[scale_info(skip_type_params(LogsBloomSize, ExtraDataSize))]
#[codec(mel_bound())]
pub struct ExecutionHeader<LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>> {
	pub parent_hash: H256,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
	pub logs_bloom: BoundedVec<u8, LogsBloomSize>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: BoundedVec<u8, ExtraDataSize>,
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
}

#[derive(Debug, PartialEq)]
pub enum ConvertError {
	FromCapellaPayloadToBellatrixError,
	FromExecutionPayloadToHeaderError,
}

impl<FeeRecipientSize: Get<u32>, LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>>
	TryFrom<ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>>
	for ExecutionHeader<LogsBloomSize, ExtraDataSize>
{
	type Error = ConvertError;

	fn try_from(
		execution_payload: ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>,
	) -> Result<Self, Self::Error> {
		let mut fee_recipient = [0u8; 20];
		let fee_slice = execution_payload.fee_recipient.as_slice();
		if fee_slice.len() == 20 {
			fee_recipient[0..20].copy_from_slice(&(fee_slice));
		} else {
			return Err(ConvertError::FromExecutionPayloadToHeaderError)
		}
		Ok(ExecutionHeader {
			parent_hash: execution_payload.parent_hash,
			fee_recipient: H160::from(fee_recipient),
			state_root: execution_payload.state_root,
			receipts_root: execution_payload.receipts_root,
			logs_bloom: execution_payload.logs_bloom,
			prev_randao: execution_payload.prev_randao,
			block_number: execution_payload.block_number,
			gas_used: execution_payload.gas_used,
			gas_limit: execution_payload.gas_limit,
			timestamp: execution_payload.timestamp,
			extra_data: execution_payload.extra_data,
			base_fee_per_gas: execution_payload.base_fee_per_gas,
			block_hash: execution_payload.block_hash,
			transactions_root: execution_payload.transactions_root,
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
pub struct SyncCommittee<SyncCommitteeSize: Get<u32>> {
	pub pubkeys: BoundedVec<PublicKey, SyncCommitteeSize>,
	pub aggregate_pubkey: PublicKey,
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(SyncCommitteeSize, SignatureSize))]
#[codec(mel_bound())]
pub struct SyncAggregate<SyncCommitteeSize: Get<u32>, SignatureSize: Get<u32>> {
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub sync_committee_bits: BoundedVec<u8, SyncCommitteeSize>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub sync_committee_signature: BoundedVec<u8, SignatureSize>,
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(FeeRecipientSize, LogsBloomSize, ExtraDataSize))]
#[codec(mel_bound())]
pub struct ExecutionPayload<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
> {
	pub parent_hash: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub fee_recipient: BoundedVec<u8, FeeRecipientSize>,
	pub state_root: H256,
	pub receipts_root: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub logs_bloom: BoundedVec<u8, LogsBloomSize>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub extra_data: BoundedVec<u8, ExtraDataSize>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_int_to_u256"))]
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
}

impl<FeeRecipientSize: Get<u32>, LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>>
	TryFrom<ExecutionPayloadCapella<FeeRecipientSize, LogsBloomSize, ExtraDataSize>>
	for ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>
{
	type Error = ConvertError;

	fn try_from(
		payload: ExecutionPayloadCapella<FeeRecipientSize, LogsBloomSize, ExtraDataSize>,
	) -> Result<Self, Self::Error> {
		Ok(ExecutionPayload {
			parent_hash: payload.parent_hash,
			fee_recipient: payload.fee_recipient,
			state_root: payload.state_root,
			receipts_root: payload.receipts_root,
			logs_bloom: payload.logs_bloom,
			prev_randao: payload.prev_randao,
			block_number: payload.block_number,
			gas_limit: payload.gas_limit,
			gas_used: payload.gas_used,
			timestamp: payload.timestamp,
			extra_data: payload.extra_data,
			base_fee_per_gas: payload.base_fee_per_gas,
			block_hash: payload.block_hash,
			transactions_root: payload.transactions_root,
		})
	}
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(FeeRecipientSize, LogsBloomSize, ExtraDataSize))]
#[codec(mel_bound())]
pub struct ExecutionPayloadCapella<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
> {
	pub parent_hash: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub fee_recipient: BoundedVec<u8, FeeRecipientSize>,
	pub state_root: H256,
	pub receipts_root: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub logs_bloom: BoundedVec<u8, LogsBloomSize>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_hex_to_bytes"))]
	pub extra_data: BoundedVec<u8, ExtraDataSize>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "from_int_to_u256"))]
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
	pub withdrawals_root: H256,
}

#[derive(
	Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(
	feature = "std",
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[scale_info(skip_type_params(FeeRecipientSize, LogsBloomSize, ExtraDataSize))]
#[codec(mel_bound())]
pub enum VersionedExecutionPayload<
	FeeRecipientSize: Get<u32>,
	LogsBloomSize: Get<u32>,
	ExtraDataSize: Get<u32>,
> {
	Bellatrix(ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>),
	Capella(ExecutionPayloadCapella<FeeRecipientSize, LogsBloomSize, ExtraDataSize>),
}

impl<FeeRecipientSize: Get<u32>, LogsBloomSize: Get<u32>, ExtraDataSize: Get<u32>>
	VersionedExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>
{
	pub fn to_payload(
		&self,
	) -> Result<ExecutionPayload<FeeRecipientSize, LogsBloomSize, ExtraDataSize>, ConvertError> {
		match self {
			VersionedExecutionPayload::Bellatrix(execution_payload) =>
				Ok(execution_payload.clone()),
			VersionedExecutionPayload::Capella(capella_execution_payload) =>
				capella_execution_payload.clone().try_into(),
		}
	}

	pub fn to_header(&self) -> Result<ExecutionHeader<LogsBloomSize, ExtraDataSize>, ConvertError> {
		match self {
			VersionedExecutionPayload::Bellatrix(execution_payload) =>
				execution_payload.clone().try_into(),
			VersionedExecutionPayload::Capella(capella_execution_payload) => {
				let execution_payload: ExecutionPayload<
					FeeRecipientSize,
					LogsBloomSize,
					ExtraDataSize,
				> = capella_execution_payload.clone().try_into()?;
				execution_payload.try_into()
			},
		}
	}
}

impl<S: Get<u32>, M: Get<u32>> ExecutionHeader<S, M> {
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
fn from_hex_to_bytes<'de, D, S>(deserializer: D) -> Result<BoundedVec<u8, S>, D::Error>
where
	D: Deserializer<'de>,
	S: Get<u32>,
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

	match BoundedVec::try_from(hex_bytes) {
		Ok(bounded) => return Ok(bounded),
		Err(_) => return Err(Error::custom("unable to create bounded vec")),
	};
}

#[cfg(feature = "std")]
fn from_int_to_u256<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: Deserializer<'de>,
{
	let number = u128::deserialize(deserializer)?;

	Ok(U256::from(number))
}
