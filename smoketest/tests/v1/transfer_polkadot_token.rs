use alloy::primitives::Address;
use assethub::api::polkadot_xcm::calls::TransactionApi;
use snowbridge_smoketest::{
	constants::*,
	contracts::token::Token::Transfer,
	helper::{initial_clients, wait_for_ethereum_event, AssetHubConfig},
	parachains::assethub::{
		api::runtime_types::{
			staging_xcm::v4::{
				asset::{Asset, AssetId, Assets, Fungibility},
				junction::{Junction, NetworkId},
				junctions::Junctions,
				location::Location,
			},
			xcm::{VersionedAssets, VersionedLocation},
		},
		{self},
	},
};
use subxt::OnlineClient;
use subxt_signer::sr25519::dev;

#[tokio::test]
async fn transfer_polkadot_token() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let amount: u128 = 1_000_000_000;
	let assets = VersionedAssets::V4(Assets(vec![Asset {
		id: AssetId(Location { parents: 1, interior: Junctions::Here }),
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
			key: ETHEREUM_ADDRESS.into(),
		}]),
	});

	let signer = dev::bob();

	let token_transfer_call =
		TransactionApi.reserve_transfer_assets(destination, beneficiary, assets, 0);

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success");

	let erc20_dot_address: Address = ERC20_DOT_CONTRACT.into();

	wait_for_ethereum_event::<Transfer>(ethereum_client, erc20_dot_address).await;
}
