use assethub::api::polkadot_xcm::calls::TransactionApi;
use ethers::{
	addressbook::Address,
	prelude::Middleware,
	providers::{Provider, Ws},
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{token, token::TransferFilter},
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
use std::{sync::Arc, time::Duration};
use subxt::OnlineClient;
use subxt_signer::sr25519::dev;

#[tokio::test]
async fn transfer_polkadot_token() {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let ethereum_client = Arc::new(ethereum_provider);

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let amount: u128 = 1_000_000_000;
	let assets = VersionedAssets::V3(MultiAssets(vec![MultiAsset {
		id: AssetId::Concrete(MultiLocation { parents: 1, interior: Junctions::Here }),
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
			key: ETHEREUM_ADDRESS.into(),
		}),
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
	let erc20_dot = token::Token::new(erc20_dot_address, ethereum_client.clone());

	let wait_for_blocks = 500;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut transfer_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
		if let Ok(transfers) = erc20_dot
			.event::<TransferFilter>()
			.at_block_hash(block.hash.unwrap())
			.query()
			.await
		{
			for transfer in transfers {
				println!("Transfer event found at ethereum block {:?}", block.number.unwrap());
				println!("from {:?}", transfer.from);
				println!("to {:?}", transfer.to);
				assert_eq!(transfer.value, amount.into());
				transfer_event_found = true;
			}
		}
		if transfer_event_found {
			break
		}
	}
	assert!(transfer_event_found);
}
