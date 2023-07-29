use ethers::abi::Token;
use hex_literal::hex;
use snowbridge_smoketest::parachains::{
    bridgehub, relaychain,
    relaychain::api::runtime_types::{
        pallet_xcm::pallet::Call,
        rococo_runtime::RuntimeCall,
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
};
use sp_core::{sr25519::Pair, Pair as PairT};
use subxt::{
    tx::{PairSigner, TxClient, TxPayload},
    OnlineClient, PolkadotConfig,
};

const RELAY_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
const BRIDGE_HUB_WS_URL: &str = "ws://127.0.0.1:11144";
const BRIDGE_HUB_PARA_ID: u32 = 1013;

const GATETWAY_UPGRADE_MOCK_CONTRACT: [u8; 20] = hex!("f8f7758fbcefd546eaeff7de24aff666b6228e73");
const GATETWAY_UPGRADE_MOCK_CODE_HASH: [u8; 32] =
    hex!("67726aaac6200cf26f4c7afe76a0bc90e89cc5d7e96a5217c96b2bbe96a29671");

#[tokio::test]
async fn upgrade_gateway() {
    let relaychain: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(RELAY_CHAIN_WS_URL).await.unwrap();
    let bridgehub: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(BRIDGE_HUB_WS_URL).await.unwrap();

    let sudo: Pair = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

    let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(sudo);
    let tx: TxClient<PolkadotConfig, _> = TxClient::new(relaychain);

    let ethereum_control_api = bridgehub::api::ethereum_control::calls::TransactionApi;
    let params = Some(ethers::abi::encode(&[
        Token::Uint(100.into()),
        Token::Uint(200.into()),
    ]));

    // The upgrade call
    let upgrade_call = ethereum_control_api
        .upgrade(
            GATETWAY_UPGRADE_MOCK_CONTRACT.into(),
            GATETWAY_UPGRADE_MOCK_CODE_HASH.into(),
            params,
        )
        .encode_call_data(&bridgehub.metadata())
        .expect("encoded call");

    let weight = 3000000000;
    let proof_size = 18000;

    let dest = Box::new(VersionedMultiLocation::V3(MultiLocation {
        parents: 0,
        interior: Junctions::X1(Junction::Parachain(BRIDGE_HUB_PARA_ID)),
    }));
    let message = Box::new(VersionedXcm::V3(Xcm(vec![
        Instruction::UnpaidExecution {
            weight_limit: WeightLimit::Limited(Weight {
                ref_time: weight,
                proof_size: proof_size,
            }),
            check_origin: None,
        },
        Instruction::Transact {
            origin_kind: OriginKind::Superuser,
            require_weight_at_most: Weight {
                ref_time: weight,
                proof_size: proof_size,
            },
            call: DoubleEncoded {
                encoded: upgrade_call,
            },
        },
    ])));

    let sudo_api = relaychain::api::sudo::calls::TransactionApi;
    let sudo_call = sudo_api.sudo(RuntimeCall::XcmPallet(Call::send { dest, message }));

    let result = tx
        .sign_and_submit_then_watch_default(&sudo_call, &signer)
        .await
        .expect("send through sudo call.")
        .wait_for_finalized_success()
        .await
        .expect("sudo call success");

    println!("{:#?}", result);

}
