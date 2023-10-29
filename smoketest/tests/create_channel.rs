use snowbridge_smoketest::{
	contracts::i_gateway::ChannelCreatedFilter, helper::*,
	parachains::bridgehub::api::ethereum_control::events::CreateChannel,
	xcm::construct_xcm_message_with_fee,
};

#[tokio::test]
async fn create_channel() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let encoded_call = construct_create_channel_call(&test_clients.bridge_hub_client)
		.await
		.expect("construct innner call.");

	let message = construct_xcm_message_with_fee(encoded_call).await;

	let result = send_xcm_transact(&test_clients.template_client, message)
		.await
		.expect("failed to send xcm transact.");

	println!(
		"xcm call issued at block hash {:?}, transaction hash {:?}",
		result.block_hash(),
		result.extrinsic_hash()
	);

	wait_for_bridgehub_event::<CreateChannel>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<ChannelCreatedFilter>(&test_clients.ethereum_client).await;
}
