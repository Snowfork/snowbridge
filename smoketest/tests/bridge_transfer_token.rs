use std::{sync::Arc, time::Duration};

use ethers::{
	prelude::*,
	providers::{Provider, Ws},
	types::Address,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	contracts::weth9::{TransferFilter, WETH9},
	parachains::{
		assethub::api::runtime_types::xcm::{
			v3::{
				junction::{Junction, NetworkId},
				junctions::Junctions,
				multiasset::{AssetId, Fungibility, MultiAsset, MultiAssets},
				multilocation::MultiLocation,
			},
			VersionedMultiAssets, VersionedMultiLocation,
		},
		assethub::{self},
	},
};
use sp_core::{sr25519::Pair, Pair as PairT};
use subxt::{tx::PairSigner, OnlineClient, PolkadotConfig};

const ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:12144";
const ETHEREUM_API: &str = "ws://localhost:8546";

const WETH_CONTRACT: [u8; 20] = hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");
const GATEWAY_PROXY_CONTRACT: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");

const DESTINATION_ADDRESS: [u8; 20] = hex!("44a57ee2f2FCcb85FDa2B0B18EBD0D8D2333700e");

#[tokio::test]
async fn bridge_transfer_token() {
	let ethereum_provider = Provider::<Ws>::connect(ETHEREUM_API)
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let ethereum_client = Arc::new(ethereum_provider);

	let weth_addr: Address = WETH_CONTRACT.into();
	let weth = WETH9::new(weth_addr, ethereum_client.clone());

	let assethub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

	let ferdie: Pair = Pair::from_string("//Ferdie", None).expect("cannot create ferdie keypair");

	let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(ferdie);

	let amount: u128 = 1_000_000_000;
	let assets = VersionedMultiAssets::V3(MultiAssets(vec![MultiAsset {
		id: AssetId::Concrete(MultiLocation {
			parents: 2,
			interior: Junctions::X3(
				Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: 15 }),
				Junction::AccountKey20 { network: None, key: GATEWAY_PROXY_CONTRACT.into() },
				Junction::AccountKey20 { network: None, key: WETH_CONTRACT.into() },
			),
		}),
		fun: Fungibility::Fungible(amount),
	}]));
	let destination = VersionedMultiLocation::V3(MultiLocation {
		parents: 2,
		interior: Junctions::X2(
			Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: 15 }),
			Junction::AccountKey20 { network: None, key: DESTINATION_ADDRESS.into() },
		),
	});

	let bridge_transfer_api = assethub::api::bridge_transfer::calls::TransactionApi;
	let bridge_transfer_call = bridge_transfer_api.transfer_asset_via_bridge(assets, destination);

	let result = assethub
		.tx()
		.sign_and_submit_then_watch_default(&bridge_transfer_call, &signer)
		.await
		.expect("send through call.")
		.wait_for_finalized_success()
		.await
		.expect("call success");

	println!("bridge_transfer call issued at assethub block hash {:?}", result.block_hash());

	let wait_for_blocks = 50;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut transfer_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for transfer event", block.number.unwrap());
		if let Ok(transfers) =
			weth.event::<TransferFilter>().at_block_hash(block.hash.unwrap()).query().await
		{
			for transfer in transfers {
				println!("Transfer event found at ethereum block {:?}", block.number.unwrap());
				assert_eq!(transfer.src, DESTINATION_ADDRESS.into());
				assert_eq!(transfer.dst, DESTINATION_ADDRESS.into());
				assert_eq!(transfer.wad, amount.into());
				transfer_event_found = true;
			}
		}
		if transfer_event_found {
			break;
		}
	}
	assert!(transfer_event_found);
}
