use ethers::{core::types::Address, prelude::U256, utils::parse_units};
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
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());

	let token: Address = ERC20_DOT_CONTRACT.into();

	let destination_fee = 400_000_000;
	let fee: U256 = parse_units("0.01", "ether").unwrap().into();

	let amount = 500_000_000;

	let receipt = gateway
		.send_token(
			token,
			ASSET_HUB_PARA_ID,
			i_gateway::MultiAddress { kind: 1, data: (*BOB_PUBLIC).into() },
			destination_fee,
			amount,
		)
		.value(fee)
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
