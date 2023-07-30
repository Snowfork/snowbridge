use std::{sync::Arc, time::Duration};

use ethers::{
	abi::Token,
	prelude::*,
	providers::{Provider, Ws},
	types::Address,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	contracts::{
		gateway_upgrade_mock::{self, InitializedFilter},
		i_gateway::{self, UpgradedFilter},
	},
	parachains::{
		assethub::{self, api::ethereum_control},
		assethub::api::runtime_types::{
			pallet_xcm::pallet::Call,
			sp_weights::weight_v2::Weight,
			xcm::{
				double_encoded::DoubleEncoded,
				v2::OriginKind,
				v3::{
					junction::Junction, junctions::Junctions, multilocation::MultiLocation,
					Instruction, WeightLimit, Xcm,
				},
				VersionedMultiLocation, VersionedXcm,
			},
		},
	},
};
use sp_core::{blake2_256, sr25519::Pair, Pair as PairT};
use subxt::{
	tx::{PairSigner, TxPayload},
	OnlineClient, PolkadotConfig,
};

const ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:12144";
const ETHEREUM_API: &str = "ws://localhost:8546";
const RELAY_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
const BRIDGE_HUB_WS_URL: &str = "ws://127.0.0.1:11144";
const BRIDGE_HUB_PARA_ID: u32 = 1013;

const GATEWAY_PROXY_CONTRACT: &str = "0xEDa338E4dC46038493b885327842fD3E301CaB39";
const GATETWAY_UPGRADE_MOCK_CONTRACT: [u8; 20] = hex!("f8f7758fbcefd546eaeff7de24aff666b6228e73");
const GATETWAY_UPGRADE_MOCK_CODE_HASH: [u8; 32] =
	hex!("c8168aa7d2c90c399ac40b4f069bae8189ad263c8d6a9b1fc681fc05d31c8425");

#[tokio::test]
async fn bridge_transfer_token() {
	let ethereum_provider = Provider::<Ws>::connect(ETHEREUM_API)
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let ethereum_client = Arc::new(ethereum_provider);

	let gateway_addr = GATEWAY_PROXY_CONTRACT.parse::<Address>().unwrap();
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());

	let assethub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

	let ferdie: Pair = Pair::from_string("//Ferdie", None).expect("cannot create ferdie keypair");

	let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(ferdie);

    let assets: assethub::api::runtime_types::xcm::VersionedMultiAssets = todo!();
    let destination: VersionedMultiLocation = todo!();
	let bridge_transfer_api = assethub::api::bridge_transfer::calls::TransactionApi;
	let bridge_transfer_call = bridge_transfer_api.transfer_asset_via_bridge(assets, destination)

	let result = assethub
		.tx()
		.sign_and_submit_then_watch_default(&bridge_transfer_call, &signer)
		.await
		.expect("send through call.")
		.wait_for_finalized_success()
		.await
		.expect("call success");

	println!("bridge_transfer call issued at assethub block hash {:?}", result.block_hash());

	let wait_for_blocks = 30;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut upgrade_event_found = false;
	let mut initialize_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for upgraded event", block.number.unwrap());
		if let Ok(upgrades) = gateway
			.event::<UpgradedFilter>()
			.at_block_hash(block.hash.unwrap())
			.query()
			.await
		{
			for upgrade in upgrades {
				println!("Upgrade event found at ethereum block {:?}", block.number.unwrap());
				assert_eq!(upgrade.implementation, GATETWAY_UPGRADE_MOCK_CONTRACT.into());
				upgrade_event_found = true;
			}
			if upgrade_event_found {
				if let Ok(initializes) = mock_gateway
					.event::<InitializedFilter>()
					.at_block_hash(block.hash.unwrap())
					.query()
					.await
				{
					for initialize in initializes {
						println!(
							"Initialize event found at ethereum block {:?}",
							block.number.unwrap()
						);
						assert_eq!(initialize.d_0, d_0.into());
						assert_eq!(initialize.d_1, d_1.into());
						initialize_event_found = true;
					}
				}
			}
		}
		if upgrade_event_found {
			break;
		}
	}
	assert!(upgrade_event_found);
	assert!(initialize_event_found);
}
