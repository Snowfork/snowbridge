use snowbridge_smoketest::{
	asset_hub_helper::{eth_location, mint_token_to},
	contracts::i_gateway_base::ForeignTokenRegisteredFilter,
	helper::*,
	helper_v2::wait_for_ethereum_event_v2,
	parachains::{
		assethub,
		assethub::api::{
			runtime_types,
			runtime_types::{bounded_collections::bounded_vec::BoundedVec, xcm::VersionedLocation},
		},
		bridgehub::api::ethereum_system_v2::events::RegisterToken,
	},
};
use subxt_signer::sr25519::dev;

const INITIAL_FUND: u128 = 3_000_000_000_000;
const FEE: u128 = 1_000_000_000;

#[tokio::test]
async fn register_pna() {
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

	type Junctions = runtime_types::staging_xcm::v4::junctions::Junctions;
	let asset = VersionedLocation::V4(runtime_types::staging_xcm::v4::location::Location {
		parents: 1,
		interior: Junctions::Here,
	});
	let metadata = runtime_types::snowbridge_core::AssetMetadata {
		name: BoundedVec(
			"wnd"
				.as_bytes()
				.to_vec()
				.iter()
				.chain([1_u8; 29].to_vec().iter())
				.map(|v| *v)
				.collect::<Vec<u8>>(),
		),
		symbol: BoundedVec(
			"wnd"
				.as_bytes()
				.to_vec()
				.iter()
				.chain([1_u8; 29].to_vec().iter())
				.map(|v| *v)
				.collect::<Vec<u8>>(),
		),
		decimals: 12,
	};

	let ethereum_system_frontend_api =
		assethub::api::snowbridge_system_frontend::calls::TransactionApi;

	let call = ethereum_system_frontend_api.register_token(asset, metadata, FEE);

	let _ = test_clients
		.asset_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&call, &signer)
		.await
		.expect("call success");

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event_v2::<ForeignTokenRegisteredFilter>(&test_clients.ethereum_client).await;
}
