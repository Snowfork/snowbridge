use snowbridge_smoketest::{
	contracts::i_gateway::ForeignTokenRegisteredFilter,
	helper::*,
	parachains::{
		bridgehub,
		bridgehub::api::{
			ethereum_system::events::RegisterToken,
			runtime_types,
			runtime_types::{
				bounded_collections::bounded_vec::BoundedVec, staging_xcm::v4::junction::NetworkId,
				xcm::VersionedLocation,
			},
		},
	},
};
use subxt_signer::sr25519::dev;

#[tokio::test]
async fn register_polkadot_token() {
	let test_clients = initial_clients().await.expect("initialize clients");

	type Junctions = runtime_types::staging_xcm::v4::junctions::Junctions;
	type Junction = runtime_types::staging_xcm::v4::junction::Junction;
	let asset = VersionedLocation::V4(runtime_types::staging_xcm::v4::location::Location {
		parents: 1,
		interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Westend)]),
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
	let call =
		bridgehub::api::ethereum_system::calls::TransactionApi.register_token(asset, metadata);

	let result = test_clients
		.bridge_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&call, &dev::bob())
		.await
		.expect("send register call.")
		.wait_for_finalized_success()
		.await
		.expect("call success");

	println!("call issued at bridgehub block hash {:?}", result.extrinsic_hash());

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<ForeignTokenRegisteredFilter>(&test_clients.ethereum_client).await;
}
