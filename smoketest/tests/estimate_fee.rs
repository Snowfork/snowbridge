use codec::{Decode, Encode};
use sp_core::{H160, H256};

use snowbridge_smoketest::{
	helper::initial_clients,
	parachains::bridgehub::api::runtime_types::{
		bp_polkadot_core::parachains::ParaId, snowbridge_core::outbound::OperatingMode,
	},
};

#[derive(Decode, Encode, Debug)]
pub struct FeeReward {
	pub fee: u128,
	pub reward: u128,
}

#[derive(Decode, Encode, Debug)]
pub struct Message {
	/// The parachain from which the message originated
	pub origin: ParaId,
	/// The stable ID for a receiving gateway contract
	pub command: Command,
}

#[derive(Decode, Encode, Debug)]
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

#[derive(Decode, Encode, Debug)]
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

#[tokio::test]
async fn estimate_fee_reward_by_command_index() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let raw = test_clients
		.bridge_hub_client
		.rpc()
		.state_call(
			"OutboundQueueApi_compute_fee_reward_by_command_index",
			Some(&2_u8.encode()), //2 is create_agent
			None,
		)
		.await
		.unwrap()
		.to_vec();
	let fee_reward = FeeReward::decode(&mut &raw[1..]).unwrap();
	println!("{:?}", fee_reward);
	assert_eq!(fee_reward.fee, 19000000000);
	assert_eq!(fee_reward.reward, 3375000000000000);
}

#[tokio::test]
async fn estimate_fee_reward() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let command = Command::CreateAgent { agent_id: Default::default() };
	let message = Message { origin: ParaId(1000), command };
	let raw = test_clients
		.bridge_hub_client
		.rpc()
		.state_call("OutboundQueueApi_compute_fee_reward", Some(&message.encode()), None)
		.await
		.unwrap()
		.to_vec();
	let fee_reward = FeeReward::decode(&mut &raw[1..]).unwrap();
	println!("{:?}", fee_reward);
	assert_eq!(fee_reward.fee, 19000000000);
	assert_eq!(fee_reward.reward, 3375000000000000);
}
