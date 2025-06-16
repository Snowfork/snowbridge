use snowbridge_smoketest::{
	constants::{ETHEREUM_CHAIN_ID, GATEWAY_PROXY_CONTRACT},
	contracts::i_gateway_base::IGatewayBase::ForeignTokenRegistered,
	helper::*,
	parachains::{
		assethub,
		assethub::api::{
			runtime_types,
			runtime_types::{
				bounded_collections::bounded_vec::BoundedVec,
				staging_xcm::v5::{
					asset::{Asset, AssetId, Fungibility::Fungible},
					junction::{Junction, NetworkId},
					location::Location,
				},
				xcm::VersionedLocation,
			},
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

	let fee_asset_location = Location {
		parents: 2,
		interior: runtime_types::staging_xcm::v5::junctions::Junctions::X1([
			Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
		]),
	};
	let fee_asset = Asset { id: AssetId(fee_asset_location.clone()), fun: Fungible(10000) };

	let mut encoded_call = Vec::new();
	ethereum_system_frontend_api
		.register_token(asset, metadata, fee_asset)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_call)
		.expect("encoded call");

	governance_assethub_call_from_relay_chain(encoded_call)
		.await
		.expect("register pna");

	wait_for_bridgehub_event::<RegisterToken>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<ForeignTokenRegistered>(
		test_clients.ethereum_client,
		(*GATEWAY_PROXY_CONTRACT).into(),
	)
	.await;
}
