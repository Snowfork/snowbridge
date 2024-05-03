use snowbridge_smoketest::{
	contracts::i_gateway::ForeignTokenRegisteredFilter, helper::*,
	parachains::bridgehub::api::ethereum_system::events::RegisterToken,
};

#[tokio::test]
async fn register_polkadot_token() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let encoded_call = construct_register_relay_token_call(&test_clients.bridge_hub_client)
		.await
		.expect("construct inner call.");

	governance_bridgehub_call_from_relay_chain(encoded_call)
		.await
		.expect("set token fees");

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<ForeignTokenRegisteredFilter>(&test_clients.ethereum_client).await;
}
