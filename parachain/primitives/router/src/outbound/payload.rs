// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use ethabi::{self, Token};

use sp_core::{RuntimeDebug, H160, H256};
use sp_std::prelude::*;

#[derive(Clone, PartialEq, RuntimeDebug)]
pub enum Message {
	UnlockNativeTokens { token: H160, recipient: H160, amount: u128 },
}

const EXECUTOR_COMMAND: H256 = H256::zero();

impl Message {
	/// Encodes the payload so that it can executed on the Ethereum side of the bridge.
	///
	/// Returns:
	/// * A stable identifier for a receiving gateway contract within Registry.sol.
	/// * The payload passed to the Gateway contract's `handle(origin, message)` method.
	pub fn encode(&self) -> Vec<u8> {
		match self {
			Self::UnlockNativeTokens { token, recipient, amount } => {
				let executor_payload = ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*token),
					Token::Address(*recipient),
					Token::Uint((*amount).into()),
				])]);
				let message = ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(EXECUTOR_COMMAND.as_ref().to_owned()),
					Token::Bytes(executor_payload),
				])]);
				message
			},
		}
	}
}
