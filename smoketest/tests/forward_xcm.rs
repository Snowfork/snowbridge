use snowbridge_smoketest::parachains::bridgehub::api::runtime_types::sp_weights::weight_v2::Weight;
use snowbridge_smoketest::parachains::bridgehub::api::runtime_types::xcm::{
    double_encoded::DoubleEncoded,
    v2::OriginKind,
    v3::{
        junction::{Junction, NetworkId},
        junctions::Junctions,
        multiasset::AssetId::Concrete,
        multiasset::Fungibility::Fungible,
        multiasset::MultiAsset,
        multiasset::MultiAssets,
        multilocation::MultiLocation,
        Instruction, MaybeErrorCode,
        WeightLimit::Unlimited,
        Xcm,
    },
    VersionedMultiLocation, VersionedXcm,
};
use snowbridge_smoketest::parachains::bridgehub::{self};

use sp_core::{sr25519::Pair, Pair as PairT};
use subxt::tx::{PairSigner, TxPayload};
use subxt::{OnlineClient, PolkadotConfig};

use snowbridge_smoketest::parachains::template;

use ethers::signers::{LocalWallet, Signer};

const BRIDGE_WS_URL: &str = "ws://127.0.0.1:11144";
const TEMPLATE_WS_URL: &str = "ws://127.0.0.1:13144";
const TEMPLATE_PARA_ID: u32 = 1001;
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";

#[tokio::test]
async fn forward_xcm() {
    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(15u64);

    let template_client: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(TEMPLATE_WS_URL).await.unwrap();

    let encoded_call = template::api::template_pallet::calls::TransactionApi
        .do_something(1)
        .encode_call_data(&template_client.metadata())
        .expect("create call");

    let bridgehub_client: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(BRIDGE_WS_URL).await.unwrap();

    let buy_execution_fee = MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: Junctions::Here,
        }),
        fun: Fungible(2_000_000_000),
    };

    let versioned_xcm = Box::new(VersionedXcm::V3(Xcm(vec![
        Instruction::UniversalOrigin(Junction::GlobalConsensus(NetworkId::Ethereum {
            chain_id: 15,
        })),
        Instruction::DescendOrigin(Junctions::X1(Junction::AccountKey20 {
            network: None,
            key: wallet.address().to_fixed_bytes(),
        })),
        Instruction::WithdrawAsset(MultiAssets {
            0: vec![MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 1,
                    interior: Junctions::Here,
                }),
                fun: Fungible(2_000_000_000),
            }],
        }),
        Instruction::BuyExecution {
            fees: buy_execution_fee,
            weight_limit: Unlimited,
        },
        Instruction::Transact {
            origin_kind: OriginKind::SovereignAccount,
            require_weight_at_most: Weight {
                ref_time: 1_000_000_000,
                proof_size: 100_000,
            },
            call: DoubleEncoded {
                encoded: encoded_call,
            },
        },
        Instruction::ExpectTransactStatus(MaybeErrorCode::Success),
    ])));

    let dest = Box::new(VersionedMultiLocation::V3(MultiLocation {
        parents: 1,
        interior: Junctions::X1(Junction::Parachain(TEMPLATE_PARA_ID)),
    }));

    let xcm_call = bridgehub::api::ethereum_inbound_queue::calls::TransactionApi::forward_xcm(
        &bridgehub::api::ethereum_inbound_queue::calls::TransactionApi,
        *dest,
        *versioned_xcm,
    );

    let owner: Pair = Pair::from_string("//Alice", None).expect("cannot create keypair");

    let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(owner);

    let result = bridgehub_client
        .tx()
        .sign_and_submit_then_watch_default(&xcm_call, &signer)
        .await
        .expect("send through xcm call.")
        .wait_for_finalized_success()
        .await
        .expect("xcm call failed");

    println!("extrinsic: {}", result.extrinsic_hash());
}
