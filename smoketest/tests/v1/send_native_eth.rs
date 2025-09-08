use alloy::primitives::{utils::parse_units, Address};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::{
		foreign_assets::events::Issued,
		runtime_types::staging_xcm::v4::{
			junction::{Junction::GlobalConsensus, NetworkId},
			junctions::Junctions::X1,
			location::Location,
		},
	},
};
use subxt::{ext::codec::Encode, utils::AccountId32};

#[cfg(feature = "legacy-v1")]
use snowbridge_smoketest::contracts::i_gateway::IGateway;
#[cfg(not(feature = "legacy-v1"))]
use snowbridge_smoketest::contracts::i_gateway_v1::IGatewayV1 as IGateway;

#[tokio::test]
async fn send_native_eth() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, *test_clients.ethereum_client);

	let eth_address: Address = [0; 20].into();

	let destination_fee = 0;
	let fee = gateway
		.quoteSendTokenFee(eth_address, ASSET_HUB_PARA_ID, destination_fee)
		.call()
		.await
		.unwrap();

	let value = parse_units("1", "ether").unwrap().get_absolute();
	// Lock tokens into vault
	let amount: u128 = value.to::<u128>();
	let transaction = gateway
		.sendToken(
			eth_address,
			ASSET_HUB_PARA_ID,
			IGateway::MultiAddress { kind: 1, data: (*SUBSTRATE_RECEIVER).into() },
			destination_fee,
			amount,
		)
		.value(fee + value)
		.gas_price(GAS_PRICE)
		.send()
		.await
		.unwrap();
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
		interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID })]),
	};
	let expected_owner: AccountId32 = (*SUBSTRATE_RECEIVER).into();

	let mut issued_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<Issued>() {
			println!("Created event found in assethub block {}.", block.number());
			let issued = issued.unwrap();
			assert_eq!(issued.asset_id.encode(), expected_asset_id.encode());
			assert_eq!(issued.owner.0, expected_owner.0);
			assert_eq!(issued.amount, amount);
			issued_event_found = true;
		}
		if issued_event_found {
			break
		}
	}
	assert!(issued_event_found)
}
