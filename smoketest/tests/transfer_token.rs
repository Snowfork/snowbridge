use assethub::api::polkadot_xcm::calls::TransactionApi;
use ethers::{
	prelude::Middleware,
	providers::{Provider, Ws},
	types::Address,
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{
		i_gateway::IGateway,
		weth9::{TransferFilter, WETH9},
	},
	helper::AssetHubConfig,
	parachains::assethub::{
		api::runtime_types::{
			staging_xcm::v3::multilocation::MultiLocation,
			xcm::{
				v3::{
					junction::{Junction, NetworkId},
					junctions::Junctions,
					multiasset::{AssetId, Fungibility, MultiAsset, MultiAssets},
				},
				VersionedAssets, VersionedLocation,
			},
		},
		{self},
	},
};
use std::{str::FromStr, sync::Arc, time::Duration};
use subxt::OnlineClient;
use subxt_signer::{sr25519, SecretUri};

#[tokio::test]
async fn transfer_token() {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let ethereum_client = Arc::new(ethereum_provider);

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = WETH9::new(weth_addr, ethereum_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, ethereum_client.clone());

	let agent_src =
		gateway.agent_of(ASSET_HUB_AGENT_ID).await.expect("could not get agent address");

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let amount: u128 = 1_000_000_000;
	let assets = VersionedAssets::V3(MultiAssets(vec![MultiAsset {
		id: AssetId::Concrete(MultiLocation {
			parents: 2,
			interior: Junctions::X2(
				Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
				Junction::AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
			),
		}),
		fun: Fungibility::Fungible(amount),
	}]));

	let destination = VersionedLocation::V3(MultiLocation {
		parents: 2,
		interior: Junctions::X1(Junction::GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})),
	});

	let beneficiary = VersionedLocation::V3(MultiLocation {
		parents: 0,
		interior: Junctions::X1(Junction::AccountKey20 {
			network: None,
			key: (*ETHEREUM_RECEIVER).into(),
		}),
	});

	let suri = SecretUri::from_str(&SUBSTRATE_KEY).expect("Parse SURI");

	let signer = sr25519::Keypair::from_uri(&suri).expect("valid keypair");

	let token_transfer_call =
		TransactionApi.reserve_transfer_assets(destination, beneficiary, assets, 0);

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success");

	let wait_for_blocks = 500;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut transfer_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
		if let Ok(transfers) =
			weth.event::<TransferFilter>().at_block_hash(block.hash.unwrap()).query().await
		{
			for transfer in transfers {
				if transfer.src.eq(&agent_src) {
					println!("Transfer event found at ethereum block {:?}", block.number.unwrap());
					assert_eq!(transfer.src, agent_src.into());
					assert_eq!(transfer.dst, (*ETHEREUM_RECEIVER).into());
					assert_eq!(transfer.wad, amount.into());
					transfer_event_found = true;
				}
			}
		}
		if transfer_event_found {
			break
		}
	}
	assert!(transfer_event_found);
}
