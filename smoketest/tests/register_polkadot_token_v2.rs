use snowbridge_smoketest::{
	contracts::i_gateway_base::ForeignTokenRegisteredFilter,
	helper::*,
	parachains::{
		bridgehub,
		bridgehub::api::{
			ethereum_system::events::RegisterToken,
			runtime_types,
			runtime_types::{bounded_collections::bounded_vec::BoundedVec, xcm::VersionedLocation},
		},
	},
};
use subxt::tx::Payload;

#[tokio::test]
async fn register_polkadot_token_v2() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let asset = VersionedLocation::V5(runtime_types::staging_xcm::v5::location::Location {
		parents: 1,
		interior: runtime_types::staging_xcm::v5::junctions::Junctions::Here,
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

	let ethereum_system_api = bridgehub::api::ethereum_system::calls::TransactionApi;

	let mut encoded = Vec::new();
	ethereum_system_api
		.register_token_v2(asset, metadata)
		.encode_call_data_to(&test_clients.bridge_hub_client.metadata(), &mut encoded)
		.expect("encoded call");

	governance_bridgehub_call_from_relay_chain(encoded)
		.await
		.expect("set token fees");

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<ForeignTokenRegisteredFilter>(&test_clients.ethereum_client).await;
}