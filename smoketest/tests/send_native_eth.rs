use ethers::{
	core::types::{Address, U256},
	utils::parse_units,
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::i_gateway,
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::{
		foreign_assets::events::Issued,
		runtime_types::{
			staging_xcm::v3::multilocation::MultiLocation,
			xcm::v3::{
				junction::{Junction::GlobalConsensus, NetworkId},
				junctions::Junctions::X1,
			},
		},
	},
};
use subxt::{ext::codec::Encode, utils::AccountId32};

#[tokio::test]
async fn send_native_eth() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());

	let eth_address: Address = [0; 20].into();

	let destination_fee = 0;
	let fee = gateway
		.quote_send_token_fee(eth_address, ASSET_HUB_PARA_ID, destination_fee)
		.call()
		.await
		.unwrap();

	let value = parse_units("1", "ether").unwrap();
	// Lock tokens into vault
	let amount: u128 = U256::from(value).low_u128();
	let receipt = gateway
		.send_token(
			eth_address,
			ASSET_HUB_PARA_ID,
			i_gateway::MultiAddress { kind: 1, data: (*SUBSTRATE_RECEIVER).into() },
			destination_fee,
			amount,
		)
		.value(fee + amount)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();

	println!(
		"receipt transaction hash: {:#?}, transaction block: {:#?}",
		hex::encode(receipt.transaction_hash),
		receipt.block_number
	);

	// Log for OutboundMessageAccepted
	let outbound_message_accepted_log = receipt.logs.last().unwrap();

	// print log for unit tests
	print_event_log_for_unit_tests(outbound_message_accepted_log);

	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let wait_for_blocks = (*WAIT_PERIOD) as usize;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_asset_id: MultiLocation = MultiLocation {
		parents: 2,
		interior: X1(GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID })),
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
			assert_eq!(issued.owner, expected_owner);
			assert_eq!(issued.amount, amount);
			issued_event_found = true;
		}
		if issued_event_found {
			break
		}
	}
	assert!(issued_event_found)
}
