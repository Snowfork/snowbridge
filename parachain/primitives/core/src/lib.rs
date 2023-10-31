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
use sp_core::H256;
use sp_runtime::traits::AccountIdConversion;

/// The ID of an agent contract
pub type AgentId = H256;
pub use operating_mode::BasicOperatingMode;

pub fn sibling_sovereign_account<T>(para_id: ParaId) -> T::AccountId
where
	T: frame_system::Config,
{
	SiblingParaId::from(para_id).into_account_truncating()
}

pub const GWEI: u128 = 1_000_000_000;
pub const METH: u128 = 1_000_000_000_000_000;
pub const ETH: u128 = 1_000_000_000_000_000_000;
