// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Helpers for implementing runtime api

use frame_support::storage::StorageStreamIter;
use snowbridge_core::outbound::{Message, SubmitError};
use snowbridge_outbound_queue_merkle_tree::{merkle_proof, MerkleProof};
use xcm::prelude::MultiAssets;

use crate::{Config, MessageLeaves, Pallet};
use snowbridge_core::outbound::OutboundQueue;

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

pub fn estimate_fee<Runtime>(message: &Message) -> Result<MultiAssets, SubmitError>
where
	Runtime: Config,
{
	Pallet::<Runtime>::estimate_fee(message)
}

pub fn estimate_fee_by_command_index<Runtime>(command_index: u8) -> Result<MultiAssets, SubmitError>
where
	Runtime: Config,
{
	let message = command_index.try_into().map_err(|_| SubmitError::EstimateFeeFailed)?;
	let fees = Pallet::<Runtime>::estimate_fee(&message)?;
	Ok(fees)
}
