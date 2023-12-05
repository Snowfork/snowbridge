// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{
	config::{EXTRA_DATA_SIZE, FEE_RECIPIENT_SIZE, LOGS_BLOOM_SIZE},
	ssz::hash_tree_root,
	ExecutionPayloadHeaderDeneb,
};
use byte_slice_cast::AsByteSlice;
use sp_core::H256;
use sp_std::{vec, vec::Vec};
use ssz_rs::{
	prelude::{List, Vector},
	Deserialize, DeserializeError, SimpleSerializeError, Sized, U256,
};
use ssz_rs_derive::SimpleSerialize as SimpleSerializeDerive;

#[derive(Default, SimpleSerializeDerive, Clone, Debug)]
pub struct SSZExecutionPayloadHeaderDeneb {
	pub parent_hash: [u8; 32],
	pub fee_recipient: Vector<u8, FEE_RECIPIENT_SIZE>,
	pub state_root: [u8; 32],
	pub receipts_root: [u8; 32],
	pub logs_bloom: Vector<u8, LOGS_BLOOM_SIZE>,
	pub prev_randao: [u8; 32],
	pub block_number: u64,
	pub gas_limit: u64,
	pub gas_used: u64,
	pub timestamp: u64,
	pub extra_data: List<u8, EXTRA_DATA_SIZE>,
	pub base_fee_per_gas: U256,
	pub block_hash: [u8; 32],
	pub transactions_root: [u8; 32],
	pub withdrawals_root: [u8; 32],
	pub blob_gas_used: u64,
	pub excess_blob_gas: u64,
}

impl TryFrom<ExecutionPayloadHeaderDeneb> for SSZExecutionPayloadHeaderDeneb {
	type Error = SimpleSerializeError;

	fn try_from(payload: ExecutionPayloadHeaderDeneb) -> Result<Self, Self::Error> {
		Ok(SSZExecutionPayloadHeaderDeneb {
			parent_hash: payload.parent_hash.to_fixed_bytes(),
			fee_recipient: Vector::<u8, FEE_RECIPIENT_SIZE>::try_from(
				payload.fee_recipient.to_fixed_bytes().to_vec(),
			)
			.expect("checked statically; qed"),
			state_root: payload.state_root.to_fixed_bytes(),
			receipts_root: payload.receipts_root.to_fixed_bytes(),
			// Logs bloom bytes size is not constrained, so here we do need to check the try_from
			// error
			logs_bloom: Vector::<u8, LOGS_BLOOM_SIZE>::try_from(payload.logs_bloom)
				.map_err(|(_, err)| err)?,
			prev_randao: payload.prev_randao.to_fixed_bytes(),
			block_number: payload.block_number,
			gas_limit: payload.gas_limit,
			gas_used: payload.gas_used,
			timestamp: payload.timestamp,
			// Extra data bytes size is not constrained, so here we do need to check the try_from
			// error
			extra_data: List::<u8, EXTRA_DATA_SIZE>::try_from(payload.extra_data)
				.map_err(|(_, err)| err)?,
			base_fee_per_gas: U256::from_bytes_le(
				payload
					.base_fee_per_gas
					.as_byte_slice()
					.try_into()
					.expect("checked in prep; qed"),
			),
			block_hash: payload.block_hash.to_fixed_bytes(),
			transactions_root: payload.transactions_root.to_fixed_bytes(),
			withdrawals_root: payload.withdrawals_root.to_fixed_bytes(),
			blob_gas_used: payload.blob_gas_used,
			excess_blob_gas: payload.excess_blob_gas,
		})
	}
}

impl ExecutionPayloadHeaderDeneb {
	pub fn hash_tree_root(&self) -> Result<H256, SimpleSerializeError> {
		hash_tree_root::<SSZExecutionPayloadHeaderDeneb>(self.clone().try_into()?)
	}
}
