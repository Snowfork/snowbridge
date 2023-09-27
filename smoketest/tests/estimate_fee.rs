use codec::{Decode, Encode};

use snowbridge_smoketest::{
	helper::initial_clients,
	parachains::bridgehub::api::runtime_types::xcm::v3::multiasset::{Fungibility, MultiAssets},
};

#[tokio::test]
async fn estimate_fee() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let raw = test_clients
		.bridge_hub_client
		.rpc()
		.state_call("OutboundQueueApi_estimate_fee_by_command_index", Some(&2_u8.encode()), None)
		.await
		.unwrap()
		.to_vec();
	let assets = MultiAssets::decode(&mut &raw[1..]).unwrap();
	println!("{:?}", assets);
	let amount: u128 = match assets.0[0].fun {
		Fungibility::Fungible(amount) => amount,
		_ => 0,
	};
	assert_eq!(amount, 19000000000);
}
