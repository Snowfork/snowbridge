// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::CompactExecutionHeader;
use codec::{Decode, Encode};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_core::{H160, H256, U256};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// ExecutionPayloadHeader
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/beacon-chain.md#executionpayloadheader
#[derive(
	Default, Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[cfg_attr(
	feature = "std",
	derive(Serialize, Deserialize),
	serde(deny_unknown_fields, bound(serialize = ""), bound(deserialize = ""))
)]
#[codec(mel_bound())]
pub struct ExecutionPayloadHeaderDeneb {
	pub parent_hash: H256,
	pub fee_recipient: H160,
	pub state_root: H256,
	pub receipts_root: H256,
	#[cfg_attr(feature = "std", serde(deserialize_with = "crate::serde_utils::from_hex_to_bytes"))]
	pub logs_bloom: Vec<u8>,
	pub prev_randao: H256,
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	#[cfg_attr(feature = "std", serde(deserialize_with = "crate::serde_utils::from_hex_to_bytes"))]
	pub extra_data: Vec<u8>,
	#[cfg_attr(feature = "std", serde(deserialize_with = "crate::serde_utils::from_int_to_u256"))]
	pub base_fee_per_gas: U256,
	pub block_hash: H256,
	pub transactions_root: H256,
	pub withdrawals_root: H256,
	pub blob_gas_used: u64,   // [New in Deneb:EIP4844]
	pub excess_blob_gas: u64, // [New in Deneb:EIP4844]
}

impl From<ExecutionPayloadHeaderDeneb> for CompactExecutionHeader {
	fn from(execution_payload: ExecutionPayloadHeaderDeneb) -> Self {
		Self {
			parent_hash: execution_payload.parent_hash,
			block_number: execution_payload.block_number,
			state_root: execution_payload.state_root,
			receipts_root: execution_payload.receipts_root,
		}
	}
}
