// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! # Polkadot Common
//!
//! Config used for the Polkadot asset hub and bridge hub runtimes.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::parameter_types;
use xcm::opaque::lts::NetworkId;

/// The pallet index of the Ethereum inbound queue pallet in the bridge hub runtime.
pub const INBOUND_QUEUE_PALLET_INDEX: u8 = 80;

parameter_types! {
	/// Network and location for the Ethereum chain.
	pub EthereumNetwork: NetworkId = NetworkId::Ethereum { chain_id: 1 };
}
