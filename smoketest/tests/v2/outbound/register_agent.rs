use snowbridge_smoketest::{
	asset_hub_helper::{eth_location, mint_token_to},
	contracts::i_gateway_base::AgentCreatedFilter,
	helper::*,
	helper_v2::wait_for_ethereum_event_v2,
	parachains::{assethub, bridgehub::api::ethereum_system_v2::events::CreateAgent},
};
use subxt_signer::sr25519::dev;

const INITIAL_FUND: u128 = 3_000_000_000_000;
const FEE: u128 = 1_000_000_000;

#[tokio::test]
async fn register_agent() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let signer = dev::bob();

	// Mint ether to sender to pay fees
	mint_token_to(
		&test_clients.asset_hub_client,
		eth_location(),
		signer.public_key().0,
		INITIAL_FUND,
	)
	.await;

	let ethereum_system_frontend_api =
		assethub::api::snowbridge_system_frontend::calls::TransactionApi;

	let call = ethereum_system_frontend_api.create_agent(FEE);

	let _ = test_clients
		.asset_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&call, &signer)
		.await
		.expect("call success");

	wait_for_bridgehub_event::<CreateAgent>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event_v2::<AgentCreatedFilter>(&test_clients.ethereum_client).await;
}
