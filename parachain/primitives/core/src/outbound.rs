use codec::{Decode, Encode};
pub use polkadot_parachain::primitives::Id as ParaId;
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256, U256};
use sp_std::{borrow::ToOwned, vec, vec::Vec};

pub type MessageHash = H256;

/// Priority for submit the message ticket to OutboundQueue
#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum Priority {
	/// Internally will submit the message to pallet_message_queue
	Normal,
	/// Internally will submit and process the message directly without waiting scheduled by
	/// pallet_message_queue
	High,
}

/// A trait for enqueueing messages for delivery to Ethereum
pub trait OutboundQueue {
	type Ticket;

	/// Validate a message
	fn validate(message: &Message) -> Result<Self::Ticket, SubmitError>;

	/// Submit the message ticket for eventual delivery to Ethereum
	fn submit(ticket: Self::Ticket, priority: Priority) -> Result<MessageHash, SubmitError>;
}

/// Default implementation of `OutboundQueue` for tests
impl OutboundQueue for () {
	type Ticket = u64;

	fn validate(message: &Message) -> Result<Self::Ticket, SubmitError> {
		Ok(0)
	}

	fn submit(ticket: Self::Ticket, priority: Priority) -> Result<MessageHash, SubmitError> {
		Ok(MessageHash::zero())
	}
}

/// Errors returned by the [`OutboundQueue`]
#[derive(Copy, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum SubmitError {
	/// Message is too large to be safely executed on Ethereum
	MessageTooLarge,
	/// The bridge has been halted for maintenance
	BridgeHalted,
	/// Messages in temp storage over-limit
	MessagesOverLimit,
	/// Message process error
	MessageProcessError,
}

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct Message {
	/// The parachain from which the message originated
	pub origin: ParaId,
	/// The stable ID for a receiving gateway contract
	pub command: Command,
}

use ethabi::Token;

#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OperatingMode {
	Normal,
	RejectingOutboundMessages,
}

/// A command which is executable by the Gateway contract on Ethereum
#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub enum Command {
	/// Execute a sub-command within an agent for a consensus system in Polkadot
	AgentExecute {
		/// The ID of the agent
		agent_id: H256,
		/// The sub-command to be executed
		command: AgentExecuteCommand,
	},
	/// Upgrade the Gateway contract
	Upgrade {
		/// Address of the new implementation contract
		impl_address: H160,
		/// Codehash of the implementation contract
		impl_code_hash: H256,
		/// Optional list of parameters to pass to initializer in the implementation contract
		params: Option<Vec<u8>>,
	},
	/// Create an agent representing a consensus system on Polkadot
	CreateAgent {
		/// The ID of the agent, derived from the `MultiLocation` of the consensus system on
		/// Polkadot
		agent_id: H256,
	},
	/// Create bidirectional messaging channel to a parachain
	CreateChannel {
		/// The ID of the parachain
		para_id: ParaId,
		/// The agent ID of the parachain
		agent_id: H256,
	},
	/// Update the configuration of a channel
	UpdateChannel {
		/// The ID of the parachain to which the channel belongs.
		para_id: ParaId,
		/// The new operating mode
		mode: OperatingMode,
		/// The new fee to charge users for outbound messaging to Polkadot
		fee: u128,
		/// The new reward to give to relayers for submitting inbound messages from Polkadot
		reward: u128,
	},
	/// Set the global operating mode of the Gateway contract
	SetOperatingMode {
		/// The new operating mode
		mode: OperatingMode,
	},
	/// Transfer ether from an agent
	TransferNativeFromAgent {
		/// The agent ID
		agent_id: H256,
		/// The recipient of the ether
		recipient: H160,
		/// The amount to transfer
		amount: u128,
	},
}

impl Command {
	/// Compute the enum variant index
	pub fn index(&self) -> u8 {
		match self {
			Command::AgentExecute { .. } => 0,
			Command::Upgrade { .. } => 1,
			Command::CreateAgent { .. } => 2,
			Command::CreateChannel { .. } => 3,
			Command::UpdateChannel { .. } => 4,
			Command::SetOperatingMode { .. } => 5,
			Command::TransferNativeFromAgent { .. } => 6,
		}
	}

	/// ABI-encode the Command.
	/// Returns a tuple of:
	/// - Index of the command
	/// - the ABI encoded command
	pub fn abi_encode(&self) -> (u8, Vec<u8>) {
		match self {
			Command::AgentExecute { agent_id, command } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Bytes(command.abi_encode()),
				])]),
			),
			Command::Upgrade { impl_address, impl_code_hash, params } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*impl_address),
					Token::FixedBytes(impl_code_hash.as_bytes().to_owned()),
					params.clone().map_or(Token::Bytes(vec![]), Token::Bytes),
				])]),
			),
			Command::CreateAgent { agent_id } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![Token::FixedBytes(
					agent_id.as_bytes().to_owned(),
				)])]),
			),
			Command::CreateChannel { para_id, agent_id } => {
				let para_id: u32 = (*para_id).into();
				(
					self.index(),
					ethabi::encode(&[Token::Tuple(vec![
						Token::Uint(U256::from(para_id)),
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
					])]),
				)
			},
			Command::UpdateChannel { para_id, mode, fee, reward } => {
				let para_id: u32 = (*para_id).into();
				(
					self.index(),
					ethabi::encode(&[Token::Tuple(vec![
						Token::Uint(U256::from(para_id)),
						Token::Uint(U256::from((*mode) as u64)),
						Token::Uint(U256::from(*fee)),
						Token::Uint(U256::from(*reward)),
					])]),
				)
			},
			Command::SetOperatingMode { mode } => (
				self.clone().index(),
				ethabi::encode(&[Token::Tuple(vec![Token::Uint(U256::from((*mode) as u64))])]),
			),
			Command::TransferNativeFromAgent { agent_id, recipient, amount } => (
				self.clone().index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])]),
			),
		}
	}
}

/// A Sub-command executable within an agent
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum AgentExecuteCommand {
	/// Transfer ERC20 tokens
	TransferToken {
		/// Address of the ERC20 token
		token: H160,
		/// The recipient of the tokens
		recipient: H160,
		/// The amount of tokens to transfer
		amount: u128,
	},
}

impl AgentExecuteCommand {
	fn index(&self) -> u8 {
		match self {
			AgentExecuteCommand::TransferToken { .. } => 0,
		}
	}

	/// ABI-encode the sub-command
	pub fn abi_encode(&self) -> Vec<u8> {
		match self {
			AgentExecuteCommand::TransferToken { token, recipient, amount } => ethabi::encode(&[
				Token::Uint(self.index().into()),
				Token::Bytes(ethabi::encode(&[
					Token::Address(*token),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])),
			]),
		}
	}
}
