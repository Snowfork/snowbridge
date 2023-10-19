use codec::{Decode, Encode, MaxEncodedLen};
use derivative::Derivative;
use ethabi::Token;
use frame_support::{
	traits::{tokens::Balance, Get},
	BoundedVec, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
pub use polkadot_parachain::primitives::Id as ParaId;
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256, U256};
use sp_std::{borrow::ToOwned, vec, vec::Vec};
use xcm::prelude::MultiLocation;

pub type MessageHash = H256;
pub type FeeAmount = u128;
pub type GasAmount = u128;
pub type GasPriceInWei = u128;

/// A trait for enqueueing messages for delivery to Ethereum
pub trait OutboundQueue {
	type Ticket: Clone;
	type Balance: Balance;

	/// Validate an outbound message and return a tuple:
	/// 1. A ticket for submitting the message
	/// 2. The delivery fee in DOT which covers the cost of execution on Ethereum
	fn validate(message: &Message) -> Result<(Self::Ticket, Self::Balance), SubmitError>;

	/// Submit the message ticket for eventual delivery to Ethereum
	fn submit(ticket: Self::Ticket) -> Result<MessageHash, SubmitError>;
}

/// Default implementation of `OutboundQueue` for tests
impl OutboundQueue for () {
	type Ticket = u64;
	type Balance = u64;

	fn validate(message: &Message) -> Result<(Self::Ticket, Self::Balance), SubmitError> {
		Ok((0, 0))
	}

	fn submit(ticket: Self::Ticket) -> Result<MessageHash, SubmitError> {
		Ok(MessageHash::zero())
	}
}

/// SubmitError returned
#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SubmitError {
	/// Message is too large to be safely executed on Ethereum
	MessageTooLarge,
	/// The bridge has been halted for maintenance
	BridgeHalted,
}

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(
	Derivative, Encode, Decode, TypeInfo, PartialEqNoBound, EqNoBound, CloneNoBound, DebugNoBound,
)]
pub struct Message {
	/// The parachain from which the message originated
	pub origin: ParaId,
	/// The stable ID for a receiving gateway contract
	pub command: Command,
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OperatingMode {
	Normal,
	RejectingOutboundMessages,
}

/// A command which is executable by the Gateway contract on Ethereum
#[derive(
	Derivative, Encode, Decode, TypeInfo, PartialEqNoBound, EqNoBound, CloneNoBound, DebugNoBound,
)]
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
		/// Optionally invoke an initializer in the implementation contract
		initializer: Option<Initializer>,
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
	},
	/// Set the global operating mode of the Gateway contract
	SetOperatingMode {
		/// The new operating mode
		mode: OperatingMode,
	},
	/// Transfer ether from an agent contract to a recipient account
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
	pub fn abi_encode(&self) -> Vec<u8> {
		match self {
			Command::AgentExecute { agent_id, command } => ethabi::encode(&[Token::Tuple(vec![
				Token::FixedBytes(agent_id.as_bytes().to_owned()),
				Token::Bytes(command.abi_encode()),
			])]),
			Command::Upgrade { impl_address, impl_code_hash, initializer, .. } =>
				ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*impl_address),
					Token::FixedBytes(impl_code_hash.as_bytes().to_owned()),
					initializer.clone().map_or(Token::Bytes(vec![]), |i| Token::Bytes(i.params)),
				])]),
			Command::CreateAgent { agent_id } =>
				ethabi::encode(&[Token::Tuple(vec![Token::FixedBytes(
					agent_id.as_bytes().to_owned(),
				)])]),
			Command::CreateChannel { para_id, agent_id } => {
				let para_id: u32 = (*para_id).into();
				ethabi::encode(&[Token::Tuple(vec![
					Token::Uint(U256::from(para_id)),
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
				])])
			},
			Command::UpdateChannel { para_id, mode, fee } => {
				let para_id: u32 = (*para_id).into();
				ethabi::encode(&[Token::Tuple(vec![
					Token::Uint(U256::from(para_id)),
					Token::Uint(U256::from((*mode) as u64)),
					Token::Uint(U256::from(*fee)),
				])])
			},
			Command::SetOperatingMode { mode } =>
				ethabi::encode(&[Token::Tuple(vec![Token::Uint(U256::from((*mode) as u64))])]),
			Command::TransferNativeFromAgent { agent_id, recipient, amount } =>
				ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])]),
		}
	}
}

/// Representation of a call to the initializer of an implementation contract.
/// The initializer has the following ABI signature: `initialize(bytes)`.
#[derive(Encode, Decode, TypeInfo, PartialEqNoBound, EqNoBound, CloneNoBound, DebugNoBound)]
pub struct Initializer {
	/// ABI-encoded params of type `bytes` to pass to the initializer
	pub params: Vec<u8>,
	/// The initializer is allowed to consume this much gas at most.
	pub maximum_required_gas: u64,
}

pub trait GasMeter {
	/// The maximum base amount of gas used in submitting and verifying a message, before the
	/// message payload is dispatched
	const MAXIMUM_BASE_GAS: u64;

	/// Measures the maximum amount of gas a command will require to dispatch. Does not include the
	/// base cost of message submission.
	fn maximum_required(command: &Command) -> u64;
}

/// A meter that assigns a constant amount of gas for the execution of a command
pub struct ConstantGasMeter;

impl GasMeter for ConstantGasMeter {
	const MAXIMUM_BASE_GAS: u64 = 125_000;

	fn maximum_required(command: &Command) -> u64 {
		match command {
			Command::CreateAgent { .. } => 300_000,
			Command::CreateChannel { .. } => 10_0000,
			Command::UpdateChannel { .. } => 50_000,
			Command::TransferNativeFromAgent { .. } => 60_000,
			Command::SetOperatingMode { .. } => 40_000,
			Command::AgentExecute { command, .. } => match command {
				AgentExecuteCommand::TransferToken { .. } => 60_000,
			},
			Command::Upgrade { initializer, .. } => {
				let maximum_required_gas = match *initializer {
					Some(Initializer { maximum_required_gas, .. }) => maximum_required_gas,
					None => 0,
				};
				// total maximum gas must also include the gas used for updating the proxy before
				// the the initializer is called.
				100_000 + maximum_required_gas
			},
		}
	}
}

impl GasMeter for () {
	const MAXIMUM_BASE_GAS: u64 = 0;

	fn maximum_required(_: &Command) -> u64 {
		0
	}
}

/// A Sub-command executable within an agent
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
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

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound)]
pub struct OutboundQueueTicket<MaxMessageSize: Get<u32>> {
	pub id: H256,
	pub origin: ParaId,
	pub message: BoundedVec<u8, MaxMessageSize>,
}

/// Message which is awaiting processing in the MessageQueue pallet
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct EnqueuedMessage {
	/// Message ID (usually hash of message)
	pub id: H256,
	/// ID of source parachain
	pub origin: ParaId,
	/// Command to execute in the Gateway contract
	pub command: Command,
}

/// Message which has been assigned a nonce and will be committed at the end of a block
#[derive(Encode, Decode, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo)]
pub struct PreparedMessage {
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
impl From<PreparedMessage> for Token {
	fn from(x: PreparedMessage) -> Token {
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

impl From<u32> for AggregateMessageOrigin {
	fn from(value: u32) -> Self {
		AggregateMessageOrigin::Export(ExportOrigin::Sibling(value.into()))
	}
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OriginInfo {
	/// The location of this origin
	pub location: MultiLocation,
	/// The parachain hosting this origin
	pub para_id: ParaId,
	/// The deterministic ID of the agent for this origin
	pub agent_id: H256,
}

#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum ExportOrigin {
	Here,
	Sibling(ParaId),
}

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	/// Message is to be exported via a bridge
	Export(ExportOrigin),
}
