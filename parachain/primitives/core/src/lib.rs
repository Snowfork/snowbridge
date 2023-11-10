// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! # Core
//!
//! Common traits and types
#![cfg_attr(not(feature = "std"), no_std)]

pub mod inbound;
pub mod operating_mode;
pub mod outbound;
pub mod ringbuffer;

pub use polkadot_parachain_primitives::primitives::{
	Id as ParaId, IsSystem, Sibling as SiblingParaId,
};
pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};

use frame_support::traits::Contains;
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;
use xcm::prelude::{Junction::Parachain, Junctions::X1, MultiLocation};

/// The ID of an agent contract
pub type AgentId = H256;
pub use operating_mode::BasicOperatingMode;

pub fn sibling_sovereign_account<T>(para_id: ParaId) -> T::AccountId
where
	T: frame_system::Config,
{
	SiblingParaId::from(para_id).into_account_truncating()
}

pub fn sibling_sovereign_account_raw(para_id: ParaId) -> [u8; 32] {
	SiblingParaId::from(para_id).into_account_truncating()
}

pub struct AllowSiblingsOnly;
impl Contains<MultiLocation> for AllowSiblingsOnly {
	fn contains(location: &MultiLocation) -> bool {
		matches!(location, MultiLocation { parents: 1, interior: X1(Parachain(_)) })
	}
}

pub const GWEI: u128 = 1_000_000_000;
pub const METH: u128 = 1_000_000_000_000_000;
pub const ETH: u128 = 1_000_000_000_000_000_000;
