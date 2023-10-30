use codec::{Decode, Encode, MaxEncodedLen};
use ethabi::Token;
use frame_support::traits::ProcessMessage;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_arithmetic::FixedU128;
use sp_runtime::{FixedPointNumber, RuntimeDebug};
use sp_std::prelude::*;

use super::Pallet;

use snowbridge_core::{ParaId, ETH, GWEI};
pub use snowbridge_outbound_queue_merkle_tree::MerkleProof;

pub type ProcessMessageOriginOf<T> = <Pallet<T> as ProcessMessage>::Origin;

pub const LOG_TARGET: &str = "snowbridge-outbound-queue";

/// Message which has been assigned a nonce and will be committed at the end of a block
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CommittedMessage {
	/// ID of source parachain
	pub origin: ParaId,
	/// Unique nonce to prevent replaying messages
	pub nonce: u64,
	/// Command to execute in the Gateway contract
	pub command: u8,
	/// Params for the command
	pub params: Vec<u8>,
	/// Maximum gas allowed for message dispatch
	pub max_dispatch_gas: u128,
	/// Maximum gas refund for message relayer
	pub max_refund: u128,
	/// Reward in ether for delivering this message, in addition to the gas refund
	pub reward: u128,
}

/// Convert message into an ABI-encoded form for delivery to the InboundQueue contract on Ethereum
impl From<CommittedMessage> for Token {
	fn from(x: CommittedMessage) -> Token {
		Token::Tuple(vec![
			Token::Uint(u32::from(x.origin).into()),
			Token::Uint(x.nonce.into()),
			Token::Uint(x.command.into()),
			Token::Bytes(x.params.to_vec()),
			Token::Uint(x.max_dispatch_gas.into()),
			Token::Uint(x.max_refund.into()),
			Token::Uint(x.reward.into()),
		])
	}
}

pub const MAX_FEE_PER_GAS: u128 = 300 * GWEI;
pub const MAX_REWARD: u128 = ETH;
pub const MAX_EXCHANGE_RATE: FixedU128 = FixedU128::from_rational(1, 10_000);

/// Configuration for fee calculations
#[derive(
	Encode,
	Decode,
	Copy,
	Clone,
	PartialEq,
	RuntimeDebug,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
)]
pub struct FeeConfigRecord {
	/// ETH/DOT exchange rate
	pub exchange_rate: FixedU128,
	/// Ether fee per unit of gas
	pub fee_per_gas: u128,
	/// Ether reward for delivering message
	pub reward: u128,
}

impl Default for FeeConfigRecord {
	fn default() -> Self {
		Self { exchange_rate: FixedU128::saturating_from_rational(1, 1), fee_per_gas: 1, reward: 0 }
	}
}

#[derive(RuntimeDebug)]
pub struct InvalidFeeConfig;

impl FeeConfigRecord {
	pub fn validate(&self) -> Result<(), InvalidFeeConfig> {
		if self.exchange_rate < MAX_EXCHANGE_RATE {
			return Err(InvalidFeeConfig)
		}
		if self.fee_per_gas == 0 || self.fee_per_gas > MAX_FEE_PER_GAS {
			return Err(InvalidFeeConfig)
		}
		if self.reward > MAX_REWARD {
			return Err(InvalidFeeConfig)
		}
		Ok(())
	}
}
