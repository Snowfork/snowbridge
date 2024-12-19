use assethub::api::polkadot_xcm::calls::TransactionApi;
use ethers::{
	prelude::Middleware,
	providers::{Provider, Ws},
	types::Address,
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::i_gateway::{IGateway, InboundMessageDispatchedFilter},
	helper::AssetHubConfig,
	parachains::assethub::{
		self,
		api::{
			polkadot_xcm::events::Sent,
			runtime_types::{
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
		},
	},
};
use std::{str::FromStr, sync::Arc, time::Duration};
use subxt::OnlineClient;
use subxt_signer::{sr25519, SecretUri};

#[tokio::test]
async fn transfer_native_eth() {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let ethereum_client = Arc::new(ethereum_provider);

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, ethereum_client.clone());

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let amount: u128 = 1_000_000_000;
	let assets = VersionedAssets::V3(MultiAssets(vec![MultiAsset {
		id: AssetId::Concrete(MultiLocation {
			parents: 2,
			interior: Junctions::X1(Junction::GlobalConsensus(NetworkId::Ethereum {
				chain_id: ETHEREUM_CHAIN_ID,
			})),
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

	let events = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success")
		.wait_for_finalized_success()
		.await
		.expect("sucessful call");

	let message_id = events
		.find_first::<Sent>()
		.expect("xcm sent")
		.expect("xcm sent found")
		.message_id;

	let receiver: Address = (*ETHEREUM_RECEIVER).into();
	let balance_before = ethereum_client.get_balance(receiver, None).await.expect("fetch balance");

	let wait_for_blocks = 500;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut transfer_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
		if let Ok(transfers) = gateway
			.event::<InboundMessageDispatchedFilter>()
			.at_block_hash(block.hash.unwrap())
			.query()
			.await
		{
			for transfer in transfers {
				if transfer.message_id.eq(&message_id) {
					println!("Transfer event found at ethereum block {:?}", block.number.unwrap());
					assert!(transfer.success, "delivered successfully");
					transfer_event_found = true;
				}
			}
		}
		if transfer_event_found {
			break
		}
	}
	assert!(transfer_event_found);
	let balance_after = ethereum_client.get_balance(receiver, None).await.expect("fetch balance");
	assert_eq!(balance_before + amount, balance_after)
}
