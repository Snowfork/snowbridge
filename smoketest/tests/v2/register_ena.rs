use alloy::primitives::{Address, U256};
use codec::Encode;
use futures::StreamExt;
use snowbridge_smoketest::{
	asset_hub_helper::{create_asset_pool, weth_location},
	constants::*,
	contracts::{i_gateway_v2 as i_gateway, weth9},
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::foreign_assets::events::Created,
};
use subxt::utils::AccountId32;

#[tokio::test]
async fn register_ena() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;
	let assethub = *(test_clients.asset_hub_client.clone());

	create_asset_pool(&Box::new(assethub.clone())).await;

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let receipt = gateway
		.v2_registerToken(*weth.address(), 0, 1_500_000_000_000u128, 1_500_000_000_000u128)
		.value(U256::from(13_000_000_000_000u128))
		.send()
		.await
		.unwrap()
		.get_receipt()
		.await
		.expect("get receipt");

	println!(
		"receipt transaction hash: {:#?}, transaction block: {:#?}",
		hex::encode(receipt.transaction_hash),
		receipt.block_number
	);

	// Log for OutboundMessageAccepted
	let outbound_message_accepted_log = receipt.logs().last().unwrap().as_ref();

	// print log for unit tests
	print_event_log_for_unit_tests(outbound_message_accepted_log);

	assert_eq!(receipt.status(), true);

	let wait_for_blocks = (*WAIT_PERIOD) as usize;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_creator: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();
	let expected_owner: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();

	let mut created_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for created event.", block.number());

		let events = block.events().await.unwrap();
		for created in events.find::<Created>() {
			println!("Created event found in assethub block {}.", block.number());
			let created = created.unwrap();
			assert_eq!(created.asset_id.encode(), weth_location().encode());
			assert_eq!(created.creator, expected_creator);
			assert_eq!(created.owner, expected_owner);
			created_event_found = true;
		}
		if created_event_found {
			break
		}
	}
	assert!(created_event_found)
}
