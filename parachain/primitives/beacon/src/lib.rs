#![cfg_attr(not(feature = "std"), no_std)]

use scale_info::TypeInfo;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use sp_core::{H160, H256, U256};
use sp_io::hashing::keccak_256;
use snowbridge_ethereum::mpt;
use core::fmt::Formatter;
use frame_support::log;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize, Serializer, Deserializer};
#[cfg(feature = "std")]
use serde::de::Visitor;

use sp_std::fmt::Result as StdResult;

pub type Root = H256;
pub type Domain = H256;
pub type ValidatorIndex = u64;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PublicKey(pub [u8; 48]);

impl Default for PublicKey {
	fn default() -> Self {
		PublicKey([0u8; 48])
	}
}

#[cfg(feature = "std")]
impl Serialize for PublicKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		log::info!(target: "ethereum-beacon-client","💫 In serialize {:?}.", self);
		serializer.serialize_bytes(&self.0)
	}
}

struct I8Visitor;

#[cfg(feature = "std")]
impl<'de> Visitor<'de> for I8Visitor {
	type Value = PublicKey;

	fn expecting(&self, formatter: &mut Formatter) -> StdResult {
		log::info!(target: "ethereum-beacon-client","💫 In expecting.");
		formatter.write_str("an array of bytes")
	}

	fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
		let mut data = [0u8; 48];
		data[0..48].copy_from_slice(&v);
		Ok(PublicKey(data))
	}

	 fn visit_seq<V>(self, seq: V) -> Result<Self::Value, V::Error> {
		log::info!(target: "ethereum-beacon-client","💫 In visit_seq.");
	 }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for PublicKey {
	type Error;
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		println!("In deserialize");
		deserializer.deserialize_seq(I8Visitor)
	}
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

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct DepositData {
	pub pubkey: Vec<u8>,
	pub withdrawal_credentials: H256,
	pub amount: u64,
	pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Deposit {
	pub proof: Vec<H256>,
	pub data: DepositData,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Checkpoint {
	pub epoch: u64,
	pub root: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttestationData {
	pub slot: u64,
	pub index: u64,
	pub beacon_block_root: H256,
	pub source: Checkpoint,
	pub target: Checkpoint,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SignedHeader {
	pub message: crate::BeaconHeader,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ProposerSlashing {
	pub signed_header_1: SignedHeader,
	pub signed_header_2: SignedHeader,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AttesterSlashing {
	pub attestation_1: IndexedAttestation,
	pub attestation_2: IndexedAttestation,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Attestation {
	pub aggregation_bits: Vec<u8>,
	pub data: AttestationData,
    pub signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct VoluntaryExit {
	pub epoch: u64,
	pub validator_index: u64,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Eth1Data {
	pub deposit_root: H256,
	pub deposit_count: u64,
	pub block_hash: H256,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct SyncAggregate {
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
}

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExecutionPayload {
	pub parent_hash: H256,
	pub fee_recipient: Vec<u8>,
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

#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Body {
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
					return Some(keccak_256(bytes));
				}
				None
			});

		final_hash.map(|hash| (hash.into(), item_to_prove.value))
	}
}

#[cfg(test)]
mod tests {
    use crate::{PublicKey, SyncCommittee};
	use hex_literal::hex;

    #[test]
    pub fn test_deserialize() {
		//let mut pk_tmp: [u8; 48] = [0; 48];

		//pk_tmp.copy_from_slice(hex!("948bd1599c5ba61106cc3bfb5118f10fd01b8b2dca6dc5a62645ccca120c6cb3252c37c9a081e3acfa6d5e181c7aebb8").to_vec().as_slice());

		//let pk = PublicKey(pk_tmp);

		//let serialized: PublicKey = serde_json::from_slice(hex!("948bd1599c5ba61106cc3bfb5118f10fd01b8b2dca6dc5a62645ccca120c6cb3252c37c9a081e3acfa6d5e181c7aebb8").as_slice()).unwrap();

		//let mut chars = serialized.chars();
		//chars.next();
		//chars.next_back();
		//serialized = chars.as_str().to_string();

		let sync_committee = SyncCommittee{
			pubkeys: vec![
 				PublicKey(hex!("948bd1599c5ba61106cc3bfb5118f10fd01b8b2dca6dc5a62645ccca120c6cb3252c37c9a081e3acfa6d5e181c7aebb8"))
			],
			aggregate_pubkey: PublicKey(hex!("898581607ef065e15ba36aeb530eada499531284426e542c3a307df1722d72122e7846fc3d770c8f475d66cd9d5004be"))
		};

		let value = serde_json::to_value(sync_committee).unwrap();

		let serialized = serde_json::to_string(&value).unwrap();
		println!("serialized = {:?}", serialized);

    	let deserialized: SyncCommittee = serde_json::from_str(&serialized).unwrap();
    	println!("deserialized = {:?}", deserialized);
	}
}