use codec::{Decode, Encode, MaxEncodedLen};
use derivative::Derivative;
use ethabi::Token;
use frame_support::{
	traits::{ConstU32, Get},
	BoundedBTreeMap, BoundedVec, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
pub use polkadot_parachain::primitives::Id as ParaId;
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, H160, H256, U256};
use sp_runtime::{FixedU128, Percent};
use sp_std::{borrow::ToOwned, vec, vec::Vec};
use xcm::prelude::{MultiAssets, MultiLocation};

pub type MessageHash = H256;
pub type CommandIndex = u8;
pub type FeeAmount = u128;
pub type GasAmount = u128;
pub type GasPriceInWei = u128;

/// A trait for enqueueing messages for delivery to Ethereum
pub trait OutboundQueue {
	type Ticket: Clone;

	/// Validate a message
	fn validate(message: &Message) -> Result<Self::Ticket, SubmitError>;

	/// Submit the message ticket for eventual delivery to Ethereum
	fn submit(ticket: Self::Ticket) -> Result<MessageHash, SubmitError>;

	/// Estimate fee
	fn estimate_fee(message: &Message) -> Result<MultiAssets, SubmitError>;
}

/// Default implementation of `OutboundQueue` for tests
impl OutboundQueue for () {
	type Ticket = u64;

	fn validate(message: &Message) -> Result<Self::Ticket, SubmitError> {
		Ok(0)
	}

	fn submit(ticket: Self::Ticket) -> Result<MessageHash, SubmitError> {
		Ok(MessageHash::zero())
	}

	fn estimate_fee(message: &Message) -> Result<MultiAssets, SubmitError> {
		Ok(MultiAssets::default())
	}
}

/// SubmitError returned
#[derive(Derivative, Encode, Decode, TypeInfo)]
#[derivative(Clone(bound = ""), Eq(bound = ""), PartialEq(bound = ""), Debug(bound = ""))]
#[codec(encode_bound())]
#[codec(decode_bound())]
pub enum SubmitError {
	/// Message is too large to be safely executed on Ethereum
	MessageTooLarge,
	/// The bridge has been halted for maintenance
	BridgeHalted,
	/// Gas config invalid
	InvalidGas(u128),
	/// Estimate fee failed
	EstimateFeeFailed,
}

/// A message which can be accepted by the [`OutboundQueue`]
#[derive(Derivative, Encode, Decode, TypeInfo)]
#[derivative(Clone(bound = ""), Eq(bound = ""), PartialEq(bound = ""), Debug(bound = ""))]
#[codec(encode_bound())]
#[codec(decode_bound())]
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
#[derive(Derivative, Encode, Decode, TypeInfo)]
#[derivative(Clone(bound = ""), Eq(bound = ""), PartialEq(bound = ""), Debug(bound = ""))]
#[codec(encode_bound())]
#[codec(decode_bound())]
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
		self.clone().into()
	}

	/// Compute gas cost
	/// reference gas from benchmark report with some extra margin for incentive
	/// | Function Name    | min    | avg    | median | max    | # calls |
	/// | agentExecute    | 487    | 5320   | 3361   | 14074  | 4       |
	/// | createAgent    | 839    | 184709 | 237187 | 237187 | 9       |     
	/// | createChannel    | 399    | 31023  | 2829   | 75402  | 5       |
	/// | updateChannel    | 817    | 15121  | 3552   | 36762  | 5     |     
	/// | transferNativeFromAgent    | 770    | 21730  | 21730  | 42691  | 2       |
	/// | setOperatingMode    | 682    | 12838  | 13240  |  24190  | 4       |
	/// | upgrade    | 443    | 9270   | 3816   | 29004  | 4       |
	pub fn dispatch_gas(&self) -> u128 {
		match self {
			Command::CreateAgent { .. } => 300000,
			Command::CreateChannel { .. } => 100000,
			Command::UpdateChannel { .. } => 50000,
			Command::TransferNativeFromAgent { .. } => 60000,
			Command::SetOperatingMode { .. } => 40000,
			Command::AgentExecute { command, .. } => match command {
				AgentExecuteCommand::TransferToken { .. } => 30000,
			},
			// leave enough space for upgrade with arbitrary initialize logic
			Command::Upgrade { .. } => 300000,
		}
	}

	pub fn base_fee_required(&self) -> bool {
		match self {
			Command::CreateAgent { .. } => true,
			Command::CreateChannel { .. } => true,
			Command::AgentExecute { .. } => true,
			Command::UpdateChannel { .. } => true,
			Command::TransferNativeFromAgent { .. } => true,
			Command::Upgrade { .. } => false,
			Command::SetOperatingMode { .. } => false,
		}
	}

	pub fn extra_fee_required(&self) -> bool {
		match self {
			Command::CreateAgent { .. } => true,
			Command::CreateChannel { .. } => true,
			Command::AgentExecute { .. } => false,
			Command::Upgrade { .. } => false,
			Command::UpdateChannel { .. } => false,
			Command::TransferNativeFromAgent { .. } => false,
			Command::SetOperatingMode { .. } => false,
		}
	}

	/// ABI-encode the Command.
	/// Returns a tuple of:
	/// - Index of the command
	/// - the ABI encoded command
	pub fn abi_encode(&self) -> (u8, Vec<u8>, u128) {
		match self {
			Command::AgentExecute { agent_id, command } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Bytes(command.abi_encode()),
				])]),
				self.dispatch_gas(),
			),
			Command::Upgrade { impl_address, impl_code_hash, params } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*impl_address),
					Token::FixedBytes(impl_code_hash.as_bytes().to_owned()),
					params.clone().map_or(Token::Bytes(vec![]), Token::Bytes),
				])]),
				self.dispatch_gas(),
			),
			Command::CreateAgent { agent_id } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![Token::FixedBytes(
					agent_id.as_bytes().to_owned(),
				)])]),
				self.dispatch_gas(),
			),
			Command::CreateChannel { para_id, agent_id } => {
				let para_id: u32 = (*para_id).into();
				(
					self.index(),
					ethabi::encode(&[Token::Tuple(vec![
						Token::Uint(U256::from(para_id)),
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
					])]),
					self.dispatch_gas(),
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
					self.dispatch_gas(),
				)
			},
			Command::SetOperatingMode { mode } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![Token::Uint(U256::from((*mode) as u64))])]),
				self.dispatch_gas(),
			),
			Command::TransferNativeFromAgent { agent_id, recipient, amount } => (
				self.index(),
				ethabi::encode(&[Token::Tuple(vec![
					Token::FixedBytes(agent_id.as_bytes().to_owned()),
					Token::Address(*recipient),
					Token::Uint(U256::from(*amount)),
				])]),
				self.dispatch_gas(),
			),
		}
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

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	#[codec(index = 0)]
	Parachain(ParaId),
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
	pub dispatch_gas: u128,
	/// Reward in ether for delivering this message
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
			Token::Uint(x.dispatch_gas.into()),
			Token::Uint(x.reward.into()),
		])
	}
}

impl From<u32> for AggregateMessageOrigin {
	fn from(value: u32) -> Self {
		AggregateMessageOrigin::Parachain(value.into())
	}
}

#[derive(
	Encode, Decode, Clone, Default, RuntimeDebug, PartialEqNoBound, TypeInfo, MaxEncodedLen,
)]
pub struct DispatchGasRange {
	pub min: GasAmount,
	pub max: GasAmount,
}

/// The fee config for outbound message
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEqNoBound, TypeInfo, MaxEncodedLen)]
pub struct OutboundFeeConfig {
	/// base fee to cover the processing costs on BridgeHub in DOT
	pub base_fee: Option<FeeAmount>,
	/// gas cost for each command
	pub command_gas_map: Option<
		BoundedBTreeMap<CommandIndex, GasAmount, ConstU32<{ CommandIndex::max_value() as u32 }>>,
	>,
	/// gas range applies for all commands
	pub gas_range: Option<DispatchGasRange>,
	/// gas price in Wei from https://etherscan.io/gastracker
	pub gas_price: Option<GasPriceInWei>,
	/// swap ratio for Ether->DOT from https://www.coingecko.com/en/coins/polkadot/eth with precision difference between Ether->DOT(18->10)
	pub swap_ratio: Option<FixedU128>,
	/// ratio from extra_fee as reward for message relay
	pub reward_ratio: Option<Percent>,
}

impl Default for OutboundFeeConfig {
	fn default() -> Self {
		OutboundFeeConfig {
			base_fee: Some(1_000_000_000),
			gas_price: Some(15_000_000_000),
			swap_ratio: Some(FixedU128::from_rational(400, 100_000_000)),
			reward_ratio: Some(Percent::from_percent(75)),
			command_gas_map: None,
			gas_range: Some(DispatchGasRange { min: 20000, max: 5000000 }),
		}
	}
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum CommandConvertError {
	/// Unsupported
	Unsupported,
}

impl From<Command> for CommandIndex {
	fn from(command: Command) -> Self {
		match command {
			Command::AgentExecute { .. } => 0,
			Command::Upgrade { .. } => 1,
			Command::CreateAgent { .. } => 2,
			Command::CreateChannel { .. } => 3,
			Command::UpdateChannel { .. } => 4,
			Command::SetOperatingMode { .. } => 5,
			Command::TransferNativeFromAgent { .. } => 6,
		}
	}
}

impl TryFrom<CommandIndex> for Command {
	type Error = CommandConvertError;

	fn try_from(value: CommandIndex) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Command::AgentExecute {
				agent_id: Default::default(),
				command: AgentExecuteCommand::TransferToken {
					token: Default::default(),
					recipient: Default::default(),
					amount: 0,
				},
			}),
			2 => Ok(Command::CreateAgent { agent_id: Default::default() }),
			3 => Ok(Command::CreateChannel {
				para_id: Default::default(),
				agent_id: Default::default(),
			}),
			_ => Err(CommandConvertError::Unsupported),
		}
	}
}

impl TryFrom<CommandIndex> for Message {
	type Error = CommandConvertError;

	fn try_from(command_index: CommandIndex) -> Result<Self, Self::Error> {
		let command = TryFrom::try_from(command_index)?;
		let message = Message { origin: Default::default(), command };
		Ok(message)
	}
}

#[derive(Copy, Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OriginInfo {
	pub agent_id: H256,
	pub para_id: ParaId,
	pub location: MultiLocation,
}
