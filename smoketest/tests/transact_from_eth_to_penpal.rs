use ethers::{core::types::Address, prelude::U256, types::Bytes};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	constants::*,
	contracts::{gateway_upgrade_mock::TransactMessage, i_gateway, shared_types::Weight},
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::penpal::api::system::events::Remarked,
};

#[tokio::test]
async fn transact_from_eth_to_penpal() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let penpal_client = *(test_clients.penpal_client.clone());

	let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());

	let message = TransactMessage {
		origin_kind: 0,
		fee: 40_000_000_000,
		weight_at_most: Weight { ref_time: 40_000_000, proof_size: 8_000 },
		//system.remark
		call: Bytes::from(hex!("00071468656c6c6f").to_vec()),
	};

	let receipt = gateway
		.transact(PENPAL_PARA_ID, message)
		.value::<U256>(100_000_000_000_000_000_u128.into())
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

	let wait_for_blocks = 50;
	let mut blocks = penpal_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let mut event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling penpal block {} for system remark event.", block.number());

		let events = block.events().await.unwrap();
		for _ in events.find::<Remarked>() {
			println!("Remarked event found in penpal block {}.", block.number());
			event_found = true;
		}
		if event_found {
			break
		}
	}
	assert!(event_found)
}
