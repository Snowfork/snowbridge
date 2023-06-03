


use ethabi::{self, Token};

use hex_literal::hex;
use snowbridge_core::{ContractId};
use sp_core::{RuntimeDebug, H160};
use sp_std::{prelude::*};



#[derive(Clone, PartialEq, RuntimeDebug)]
pub enum Message {
	UpgradeProxy(UpgradeProxyMessage),
	NativeTokens(NativeTokensMessage),
}

/// A message to be sent to `UpgradeProxy.sol`.
#[derive(Clone, PartialEq, RuntimeDebug)]
pub enum UpgradeProxyMessage {
	/// Run an upgrade task with elevated privileges
	Upgrade {
		/// The address of the upgrader contract which implements `UpgradeTask.sol`.
		upgrade_task: H160,
	},
}

/// A message to be sent to `NativeTokens.sol`.
#[derive(Clone, PartialEq, RuntimeDebug)]
pub enum NativeTokensMessage {
	/// Release locked collateral for ERC20 token identified by `asset` back to the specified
	/// `destination` account
	Unlock {
		/// ERC20 token address
		asset: H160,
		/// Account which will receive the tokens
		destination: H160,
		/// Amount of tokens to release
		amount: u128,
	},
}

impl Message {
	/// Encodes the payload so that it can executed on the Ethereum side of the bridge.
	///
	/// Returns:
	/// * A stable identifier for a receiving gateway contract within Registry.sol.
	/// * The payload passed to the Gateway contract's `handle(origin, message)` method.
	pub fn encode(&self) -> (ContractId, Vec<u8>) {
		match self {
			Self::UpgradeProxy(UpgradeProxyMessage::Upgrade { upgrade_task }) => {
				let inner = ethabi::encode(&[Token::Tuple(vec![Token::Address(*upgrade_task)])]);
				let message = ethabi::encode(&[Token::Tuple(vec![
					Token::Uint(0.into()), // Upgrade action = 0
					Token::Bytes(inner),
				])]);
				(
					// keccak256("UpgradeProxy")
					hex!["44bef07c29162ad04096f5cbe78ca2df62dffe97cea85825f08d13319e13f34a"].into(),
					message,
				)
			},
			Self::NativeTokens(NativeTokensMessage::Unlock { asset, destination, amount }) => {
				let inner = ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*asset),
					Token::Address(*destination),
					Token::Uint((*amount).into()),
				])]);
				let message = ethabi::encode(&[Token::Tuple(vec![
					Token::Uint(0.into()), // Unlock action = 0
					Token::Bytes(inner),
				])]);
				(
					// keccak256("NativeTokens")
					hex!["1d0761c5c76335b59fce9e8070a90d04470a4d5806c9814b73032db3dbb843ea"].into(),
					message,
				)
			},
		}
	}
}
