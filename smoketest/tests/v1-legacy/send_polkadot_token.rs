use alloy::primitives::{utils::parse_units, Address, U256};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::i_gateway,
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::balances::events::Minted,
};
use subxt::utils::AccountId32;

#[tokio::test]
async fn send_polkadot_token() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGateway::new(gateway_addr, test_clients.ethereum_client);

	let token: Address = (*ERC20_DOT_CONTRACT).into();

	let destination_fee = 400_000_000;
	let fee: U256 = parse_units("0.01", "ether").unwrap().get_absolute();

	let amount = 500_000_000;

	let transaction = gateway
		.sendToken(
			token,
			ASSET_HUB_PARA_ID,
			i_gateway::IGateway::MultiAddress { kind: 1, data: (*BOB_PUBLIC).into() },
			destination_fee,
			amount,
		)
		.value(fee)
		.send()
		.await
		.unwrap();
	let receipt = transaction.get_receipt().await.expect("get receipt");

	println!(
		"receipt transaction hash: {:#?}, transaction block: {:#?}",
		hex::encode(receipt.transaction_hash),
		receipt.block_number
	);

	// print log for unit tests
	print_event_log_for_unit_tests(receipt.logs().first().unwrap().as_ref());

	assert_eq!(receipt.status(), true);

	let wait_for_blocks = 500;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_owner: AccountId32 = (*BOB_PUBLIC).into();

	let mut event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for mint event.", block.number());

		let events = block.events().await.unwrap();
		for event_wrapped in events.find::<Minted>() {
			println!("event found in block {}.", block.number());
			let event = event_wrapped.unwrap();
			assert_eq!(event.who, expected_owner);
			event_found = true;
		}
		if event_found {
			break
		}
	}
	assert!(event_found)
}
