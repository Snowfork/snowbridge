use ethers::{
    providers::{Provider, Ws},
    types::Address,
};
use std::{sync::Arc, time::Duration};
use snowbridge_smoketest::contracts::weth9::WETH9;
use subxt::{
    tx::{PairSigner},
    OnlineClient, PolkadotConfig,
};
use snowbridge_smoketest::constants::{ASSET_HUB_WS_URL, ETHEREUM_API, GATEWAY_PROXY_CONTRACT, WETH_CONTRACT};
use sp_core::{sr25519::Pair, Pair as PairT};
use snowbridge_smoketest::{
    contracts::weth9::{TransferFilter},
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
use hex_literal::hex;
use sp_core::bytes::to_hex;
use subxt::tx::TxPayload;
use assethub::api::bridge_transfer::calls::TransactionApi;
use snowbridge_smoketest::helper::{AssetHubConfig, TemplateConfig};

const DESTINATION_ADDRESS: [u8; 20] = hex!("44a57ee2f2FCcb85FDa2B0B18EBD0D8D2333700e");

#[tokio::test]
async fn transfer_token() {
    let ethereum_provider = Provider::<Ws>::connect(ETHEREUM_API)
        .await
        .unwrap()
        .interval(Duration::from_millis(10u64));


    let ethereum_client = Arc::new(ethereum_provider);

    let weth_addr: Address = WETH_CONTRACT.into();
    let weth = WETH9::new(weth_addr, ethereum_client.clone());

    let assethub: OnlineClient<AssetHubConfig> =
        OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

    let keypair: Pair = Pair::from_string("//Ferdie", None).expect("cannot create keypair");

    let signer: PairSigner<AssetHubConfig, _> = PairSigner::new(keypair);

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
    /*
    let destination = VersionedMultiLocation::V3(MultiLocation {
        parents: 1,
        interior: Junctions::X2(
            Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: 15 }),
            Junction::AccountKey20 { network: None, key: DESTINATION_ADDRESS.into() },
        ),
    });*/

    let destination = VersionedMultiLocation::V3(MultiLocation {
        parents: 1,
        interior: Junctions::X2(
            Junction::GlobalConsensus(NetworkId::Ethereum { chain_id: 15 }),
            //Junction::Parachain(1013),
            Junction::AccountKey20 { network: Some(NetworkId::Ethereum { chain_id: 15 }), key: DESTINATION_ADDRESS.into() },
        ),
    });

    let bridge_transfer_call = TransactionApi.transfer_asset_via_bridge(assets, destination);

   // let calldata = assethub::api::system::calls::TransactionApi.remark(String::from("Hello, world!").into_bytes());

    let calldata = hex::encode(bridge_transfer_call.encode_call_data(&assethub.metadata()).unwrap());
    println!("Encoded {:?}", calldata);

    let result = assethub
        .tx()
        .sign_and_submit_then_watch_default(&bridge_transfer_call, &signer)
        .await
        .expect("send through call.")
        .wait_for_finalized_success()
        .await
        .expect("call success");

    println!("bridge_transfer call issued at assethub block hash {:?}", result.block_hash());
}
