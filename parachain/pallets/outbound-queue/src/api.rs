// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Helpers for implementing runtime api

use crate::{Config, MessageLeaves, Pallet};
use frame_support::storage::StorageStreamIter;
use snowbridge_core::outbound::{FeeAmount, Message, SubmitError};
use snowbridge_outbound_queue_merkle_tree::{merkle_proof, MerkleProof};

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

pub fn compute_fee_reward<Runtime>(message: &Message) -> Result<(FeeAmount, FeeAmount), SubmitError>
where
	Runtime: Config,
{
	let fee_reward = Pallet::<Runtime>::compute_fee_reward(&message.command)?;
	Ok(fee_reward)
}
