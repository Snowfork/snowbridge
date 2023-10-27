// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::tokens::Balance as BalanceT;
use snowbridge_core::outbound::Message;
use snowbridge_outbound_queue_merkle_tree::MerkleProof;

sp_api::decl_runtime_apis! {
	pub trait OutboundQueueApi<Balance> where Balance: BalanceT
	{
		fn prove_message(leaf_index: u64) -> Option<MerkleProof>;

		fn calculate_fee(message: Message) -> Option<Balance>;
	}
}
