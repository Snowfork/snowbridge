// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Helpers for implementing runtime api

use crate::{Config, MessageLeaves};
use frame_support::storage::StorageStreamIter;
use snowbridge_core::outbound::{Fees, Message, OutboundQueue};
use snowbridge_outbound_queue_merkle_tree::{merkle_proof, MerkleProof};
use sp_arithmetic::traits::SaturatedConversion;

pub fn prove_message<Runtime>(leaf_index: u64) -> Option<MerkleProof>
where
	Runtime: Config,
{
	if !MessageLeaves::<Runtime>::exists() {
		return None
	}
	let proof = merkle_proof::<<Runtime as Config>::Hashing, _>(
		MessageLeaves::<Runtime>::stream_iter(),
		leaf_index,
	);
	Some(proof)
}

pub fn calculate_fee<Runtime>(message: Message) -> Option<Fees<u128>>
where
	Runtime: Config,
{
	let fee = crate::Pallet::<Runtime>::validate(&message).ok()?.1;
	Some(Fees::<u128> {
		base: fee.base.saturated_into::<u128>(),
		delivery: fee.delivery.saturated_into::<u128>(),
	})
}
