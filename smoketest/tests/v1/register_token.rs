use alloy::primitives::Address;
use codec::Encode;
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::weth9,
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::{
		foreign_assets::events::Created,
		runtime_types::staging_xcm::v4::{
			junction::{
				Junction::{AccountKey20, GlobalConsensus},
				NetworkId,
			},
			junctions::Junctions::X2,
			location::Location,
		},
	},
};
use subxt::utils::AccountId32;

#[cfg(feature = "legacy-v1")]
use snowbridge_smoketest::contracts::i_gateway::IGateway;
#[cfg(not(feature = "legacy-v1"))]
use snowbridge_smoketest::contracts::i_gateway_v1::IGatewayV1 as IGateway;

#[tokio::test]
async fn register_token() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let fee = gateway.quoteRegisterTokenFee().call().await.unwrap();

	let transaction = gateway
		.registerToken(*weth.address())
		.value(fee)
		.gas_price(GAS_PRICE)
		.send()
		.await
		.expect("send token");
	let receipt = transaction.get_receipt().await.expect("get receipt");

	println!("receipt transaction hash: {:#?}", hex::encode(receipt.transaction_hash));

	// print log for unit tests
	print_event_log_for_unit_tests(receipt.logs().first().unwrap().as_ref());

	assert_eq!(receipt.status(), true);

	let wait_for_blocks = (*WAIT_PERIOD) as usize;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_asset_id: Location = Location {
		parents: 2,
		interior: X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	};
	let expected_creator: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();
	let expected_owner: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();

	let mut created_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for created event.", block.number());

		let events = block.events().await.unwrap();
		for created in events.find::<Created>() {
			println!("Created event found in assethub block {}.", block.number());
			let created = created.unwrap();
			assert_eq!(created.asset_id.encode(), expected_asset_id.encode());
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
