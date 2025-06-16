use alloy::{
	primitives::{Address, U256},
	providers::Provider,
};
use assethub::api::polkadot_xcm::calls::TransactionApi;
use snowbridge_smoketest::{
	constants::*,
	helper::{initial_clients, wait_for_ethereum_event, AssetHubConfig},
	parachains::assethub::{
		self,
		api::runtime_types::{
			staging_xcm::v4::{
				asset::{Asset, AssetId, Assets, Fungibility},
				junction::{Junction, NetworkId},
				junctions::Junctions,
				location::Location,
			},
			xcm::{VersionedAssets, VersionedLocation},
		},
	},
};
use std::str::FromStr;
use subxt::OnlineClient;
use subxt_signer::{sr25519, SecretUri};

#[cfg(feature = "legacy-v1")]
use snowbridge_smoketest::contracts::i_gateway::IGateway;
#[cfg(not(feature = "legacy-v1"))]
use snowbridge_smoketest::contracts::i_gateway_v1::IGatewayV1 as IGateway;

#[tokio::test]
async fn transfer_native_eth() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let ethereum_client = test_clients.ethereum_client;

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let amount: u128 = 1_000_000_000;
	let assets = VersionedAssets::V4(Assets(vec![Asset {
		id: AssetId(Location {
			parents: 2,
			interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
				chain_id: ETHEREUM_CHAIN_ID,
			})]),
		}),
		fun: Fungibility::Fungible(amount),
	}]));

	let destination = VersionedLocation::V4(Location {
		parents: 2,
		interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	});

	let beneficiary = VersionedLocation::V4(Location {
		parents: 0,
		interior: Junctions::X1([Junction::AccountKey20 {
			network: None,
			key: (*ETHEREUM_RECEIVER).into(),
		}]),
	});

	let suri = SecretUri::from_str(&SUBSTRATE_KEY).expect("Parse SURI");

	let signer = sr25519::Keypair::from_uri(&suri).expect("valid keypair");

	let token_transfer_call =
		TransactionApi.reserve_transfer_assets(destination, beneficiary, assets, 0);

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success")
		.wait_for_finalized_success()
		.await
		.expect("successful call");

	let receiver: Address = (*ETHEREUM_RECEIVER).into();
	let balance_before = ethereum_client.get_balance(receiver).await.expect("fetch balance");

	wait_for_ethereum_event::<IGateway::InboundMessageDispatched>(
		ethereum_client.clone(),
		gateway_addr,
	)
	.await;

	let balance_after = ethereum_client.get_balance(receiver).await.expect("fetch balance");
	assert_eq!(balance_before + U256::from(amount), balance_after)
}
