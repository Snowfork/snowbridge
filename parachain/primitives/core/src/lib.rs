// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! # Core
//!
//! Common traits and types

#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod inbound;
pub mod outbound;
pub mod ringbuffer;

pub use polkadot_parachain::primitives::Id as ParaId;
pub use ringbuffer::{RingBufferMap, RingBufferMapImpl};
use sp_core::H256;

/// The ID of an agent contract
pub type AgentId = H256;
/// The ID of a parachain token
pub type TokenId = H256;
