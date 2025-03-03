use snowbridge_smoketest::{
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
use subxt::tx::Payload;

#[tokio::test]
async fn register_pna() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let asset_hub_client = test_clients.asset_hub_client;
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

	let mut encoded_call = Vec::new();
	ethereum_system_frontend_api
		.register_token(asset, metadata)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_call)
		.expect("encoded call");

	governance_assethub_call_from_relay_chain(encoded_call)
		.await
		.expect("register pna");

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event_v2::<ForeignTokenRegisteredFilter>(&test_clients.ethereum_client).await;
}
