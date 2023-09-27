use codec::{Decode, Encode};

use snowbridge_smoketest::helper::initial_clients;

#[derive(Decode, Encode, Debug)]
pub struct FeeReward {
	pub fee: u128,
	pub reward: u128,
}

#[tokio::test]
async fn estimate_fee() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let raw = test_clients
		.bridge_hub_client
		.rpc()
		.state_call(
			"OutboundQueueApi_compute_fee_reward_by_command_index",
			Some(&2_u8.encode()),
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
