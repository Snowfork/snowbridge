use snowbridge_smoketest::{
	constants::*, contracts::i_gateway::InboundMessageDispatchedFilter, helper::*,
	parachains::bridgehub::api::ethereum_control::events::TransferNativeFromAgent,
	xcm::construct_xcm_message,
};

#[tokio::test]
async fn transfer_native_from_agent() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let before = get_balance(&test_clients.ethereum_signed_client, ETHEREUM_ADDRESS.into())
		.await
		.expect("get balance");

	println!("balance before: {}", before);

	const TRANSFER_AMOUNT: u128 = 1000000000;

	let message = construct_xcm_message(
		construct_transfer_native_from_agent_call(
			&test_clients.bridge_hub_client,
			ETHEREUM_ADDRESS.into(),
			TRANSFER_AMOUNT,
		)
		.await
		.expect("construct innner call."),
	);

	let result = send_xcm_transact(&test_clients.template_client, message)
		.await
		.expect("failed to send xcm transact.");

	println!(
		"xcm call issued at block hash {:?}, transaction hash {:?}",
		result.block_hash(),
		result.extrinsic_hash()
	);

	wait_for_bridgehub_event::<TransferNativeFromAgent>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<InboundMessageDispatchedFilter>(&test_clients.ethereum_client).await;

	let after = get_balance(&test_clients.ethereum_signed_client, ETHEREUM_ADDRESS.into())
		.await
		.expect("get balance");

	println!("balance after: {}", after);
	assert_eq!(before + TRANSFER_AMOUNT, after);
}
