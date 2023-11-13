use crate::ChannelId;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::PalletError;
use scale_info::TypeInfo;
use sp_arithmetic::traits::{BaseArithmetic, Unsigned};
use sp_core::{RuntimeDebug, H256};
pub use v1::{AgentExecuteCommand, Command, Initializer, Message, OperatingMode, QueuedMessage};

/// Enqueued outbound messages need to be versioned to prevent data corruption
/// or loss after forkless runtime upgrades
#[derive(Encode, Decode, TypeInfo, Clone, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(PartialEq))]
pub enum VersionedQueuedMessage {
	V1(QueuedMessage),
}

impl TryFrom<VersionedQueuedMessage> for QueuedMessage {
	type Error = ();
	fn try_from(x: VersionedQueuedMessage) -> Result<Self, Self::Error> {
		use VersionedQueuedMessage::*;
		match x {
			V1(x) => Ok(x),
		}
	}
}

impl<T: Into<QueuedMessage>> From<T> for VersionedQueuedMessage {
	fn from(x: T) -> Self {
		VersionedQueuedMessage::V1(x.into())
	}
}

mod v1 {
	use crate::ChannelId;
	use codec::{Decode, Encode};
	use ethabi::Token;
	use scale_info::TypeInfo;
	use sp_core::{RuntimeDebug, H160, H256, U256};
	use sp_std::{borrow::ToOwned, vec, vec::Vec};

	/// A message which can be accepted by implementations of [`SendMessage`]
	#[derive(Encode, Decode, TypeInfo, Clone, RuntimeDebug)]
	#[cfg_attr(feature = "std", derive(PartialEq))]
	pub struct Message {
		/// ID for this message. One will be automatically generated if not provided.
		///
		/// When this message is created from an XCM message, the ID should be extracted
		/// from the `SetTopic` instruction.
		///
		/// The ID plays no role in bridge consensus, and is purely meant for message tracing.
		pub id: Option<H256>,
		/// The message channel ID
		pub channel_id: ChannelId,
		/// The stable ID for a receiving gateway contract
		pub command: Command,
	}

	#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub enum OperatingMode {
		Normal,
		RejectingOutboundMessages,
	}

	/// A command which is executable by the Gateway contract on Ethereum
	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(PartialEq))]
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
			/// The ID of the channel
			channel_id: ChannelId,
			/// The agent ID of the parachain
			agent_id: H256,
			/// Initial operating mode
			mode: OperatingMode,
			/// The fee to charge users for outbound messaging to Polkadot
			outbound_fee: u128,
		},
		/// Update the configuration of a channel
		UpdateChannel {
			/// The ID of the channel
			channel_id: ChannelId,
			/// The new operating mode
			mode: OperatingMode,
			/// The new fee to charge users for outbound messaging to Polkadot
			outbound_fee: u128,
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
		/// Set token fees of the Gateway contract
		SetTokenTransferFees {
			/// The fee for register token
			register: u128,
			/// The fee for send token to para chain
			send: u128,
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
				Command::SetTokenTransferFees { .. } => 7,
			}
		}

		/// ABI-encode the Command.
		pub fn abi_encode(&self) -> Vec<u8> {
			match self {
				Command::AgentExecute { agent_id, command } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
						Token::Bytes(command.abi_encode()),
					])]),
				Command::Upgrade { impl_address, impl_code_hash, initializer, .. } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::Address(*impl_address),
						Token::FixedBytes(impl_code_hash.as_bytes().to_owned()),
						initializer
							.clone()
							.map_or(Token::Bytes(vec![]), |i| Token::Bytes(i.params)),
					])]),
				Command::CreateAgent { agent_id } =>
					ethabi::encode(&[Token::Tuple(vec![Token::FixedBytes(
						agent_id.as_bytes().to_owned(),
					)])]),
				Command::CreateChannel { channel_id, agent_id, mode, outbound_fee } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::FixedBytes(channel_id.as_ref().to_owned()),
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
						Token::Uint(U256::from((*mode) as u64)),
						Token::Uint(U256::from(*outbound_fee)),
					])]),
				Command::UpdateChannel { channel_id, mode, outbound_fee } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::FixedBytes(channel_id.as_ref().to_owned()),
						Token::Uint(U256::from((*mode) as u64)),
						Token::Uint(U256::from(*outbound_fee)),
					])]),
				Command::SetOperatingMode { mode } =>
					ethabi::encode(&[Token::Tuple(vec![Token::Uint(U256::from((*mode) as u64))])]),
				Command::TransferNativeFromAgent { agent_id, recipient, amount } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::FixedBytes(agent_id.as_bytes().to_owned()),
						Token::Address(*recipient),
						Token::Uint(U256::from(*amount)),
					])]),
				Command::SetTokenTransferFees { register, send } =>
					ethabi::encode(&[Token::Tuple(vec![
						Token::Uint(U256::from(*register)),
						Token::Uint(U256::from(*send)),
					])]),
			}
		}
	}

	/// Representation of a call to the initializer of an implementation contract.
	/// The initializer has the following ABI signature: `initialize(bytes)`.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	pub struct Initializer {
		/// ABI-encoded params of type `bytes` to pass to the initializer
		pub params: Vec<u8>,
		/// The initializer is allowed to consume this much gas at most.
		pub maximum_required_gas: u64,
	}

	/// A Sub-command executable within an agent
	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(PartialEq))]
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
				AgentExecuteCommand::TransferToken { token, recipient, amount } =>
					ethabi::encode(&[
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

	/// Message which is awaiting processing in the MessageQueue pallet
	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(PartialEq))]
	pub struct QueuedMessage {
		/// Message ID
		pub id: H256,
		/// Channel ID
		pub channel_id: ChannelId,
		/// Command to execute in the Gateway contract
		pub command: Command,
	}
}

#[cfg_attr(feature = "std", derive(PartialEq, Debug))]
/// Fee for delivering message
pub struct Fee<Balance>
where
	Balance: BaseArithmetic + Unsigned + Copy,
{
	/// Fee to cover cost of processing the message locally
	pub local: Balance,
	/// Fee to cover cost processing the message remotely
	pub remote: Balance,
}

impl<Balance> Fee<Balance>
where
	Balance: BaseArithmetic + Unsigned + Copy,
{
	pub fn total(&self) -> Balance {
		self.local.saturating_add(self.remote)
	}
}

impl<Balance> From<(Balance, Balance)> for Fee<Balance>
where
	Balance: BaseArithmetic + Unsigned + Copy,
{
	fn from((local, remote): (Balance, Balance)) -> Self {
		Self { local, remote }
	}
}

/// A trait for sending messages to Ethereum
pub trait SendMessage: SendMessageFeeProvider {
	type Ticket: Clone + Encode + Decode;

	/// Validate an outbound message and return a tuple:
	/// 1. Ticket for submitting the message
	/// 2. Delivery fee
	fn validate(
		message: &Message,
	) -> Result<(Self::Ticket, Fee<<Self as SendMessageFeeProvider>::Balance>), SendError>;

	/// Submit the message ticket for eventual delivery to Ethereum
	fn deliver(ticket: Self::Ticket) -> Result<H256, SendError>;
}

pub trait Ticket: Encode + Decode + Clone {
	fn message_id(&self) -> H256;
}

/// A trait for getting the local costs associated with sending a message.
pub trait SendMessageFeeProvider {
	type Balance: BaseArithmetic + Unsigned + Copy;

	/// The local component of the message processing fees in native currency
	fn local_fee() -> Self::Balance;
}

/// Reasons why sending to Ethereum could not be initiated
#[derive(Copy, Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, PalletError, TypeInfo)]
pub enum SendError {
	/// Message is too large to be safely executed on Ethereum
	MessageTooLarge,
	/// The bridge has been halted for maintenance
	Halted,
}

pub trait GasMeter {
	/// All the gas used for submitting a message to Ethereum, minus the cost of dispatching
	/// the command within the message
	const MAXIMUM_BASE_GAS: u64;

	/// Measures the maximum amount of gas a command will require to dispatch. Does not include the
	/// base cost of message submission.
	fn maximum_required(command: &Command) -> u64;
}

/// A meter that assigns a constant amount of gas for the execution of a command
///
/// The gas figures are extracted from this report:
/// > forge test --match-path test/Gateway.t.sol --gas-report
///
/// A healthy buffer is added on top of these figures to account for:
/// * The EIP-150 63/64 rule
/// * Future EVM upgrades that may increase gas cost
pub struct ConstantGasMeter;

impl GasMeter for ConstantGasMeter {
	const MAXIMUM_BASE_GAS: u64 = 125_000;

	fn maximum_required(command: &Command) -> u64 {
		match command {
			Command::CreateAgent { .. } => 275_000,
			Command::CreateChannel { .. } => 100_000,
			Command::UpdateChannel { .. } => 50_000,
			Command::TransferNativeFromAgent { .. } => 60_000,
			Command::SetOperatingMode { .. } => 40_000,
			Command::AgentExecute { command, .. } => match command {
				// Execute IERC20.transferFrom
				//
				// Worst-case assumptions are important:
				// * No gas refund for clearing storage slot of source account in ERC20 contract
				// * Assume dest account in ERC20 contract does not yet have a storage slot
				// * ERC20.transferFrom possibly does other business logic besides updating balances
				AgentExecuteCommand::TransferToken { .. } => 100_000,
			},
			Command::Upgrade { initializer, .. } => {
				let initializer_max_gas = match *initializer {
					Some(Initializer { maximum_required_gas, .. }) => maximum_required_gas,
					None => 0,
				};
				// total maximum gas must also include the gas used for updating the proxy before
				// the the initializer is called.
				50_000 + initializer_max_gas
			},
			Command::SetTokenTransferFees { .. } => 60_000,
		}
	}
}

impl GasMeter for () {
	const MAXIMUM_BASE_GAS: u64 = 1;

	fn maximum_required(_: &Command) -> u64 {
		1
	}
}

impl From<u32> for AggregateMessageOrigin {
	fn from(value: u32) -> Self {
		AggregateMessageOrigin::Snowbridge(ParaId::from(value).into())
	}
}

/// Aggregate message origin for the `MessageQueue` pallet.
#[derive(Encode, Decode, Clone, Copy, MaxEncodedLen, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AggregateMessageOrigin {
	Snowbridge(ChannelId),
}

pub const ETHER_DECIMALS: u8 = 18;
