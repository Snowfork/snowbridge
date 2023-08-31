use snowbridge_smoketest::constants::{BRIDGE_HUB_PARA_ID, DEFAULT_OPERATION_FEE};
use snowbridge_smoketest::helper::{initial_clients, wait_for_bridgehub_event};
use snowbridge_smoketest::parachains::bridgehub::api::ethereum_control::events::FeeUpdated;
use snowbridge_smoketest::parachains::{
    bridgehub::{
        api::runtime_types,
        api::runtime_types::bridge_hub_rococo_runtime::RuntimeCall as BHRuntimeCall, api::utility,
    },
    relaychain,
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
    tx::{PairSigner, TxPayload},
    PolkadotConfig,
};

#[tokio::test]
async fn configure_operation_fees() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let sudo: Pair = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

    let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(sudo);

    let update_fee_for_create_agent = BHRuntimeCall::EthereumControl(
        runtime_types::snowbridge_control::pallet::Call::update_operation_fee {
            operation: runtime_types::snowbridge_core::outbound::ControlOperation::CreateAgent,
            update_fee: DEFAULT_OPERATION_FEE,
        },
    );
    let update_fee_for_create_channel = BHRuntimeCall::EthereumControl(
        runtime_types::snowbridge_control::pallet::Call::update_operation_fee {
            operation: runtime_types::snowbridge_core::outbound::ControlOperation::CreateChannel,
            update_fee: DEFAULT_OPERATION_FEE,
        },
    );
    let update_fee_for_update_channel = BHRuntimeCall::EthereumControl(
        runtime_types::snowbridge_control::pallet::Call::update_operation_fee {
            operation: runtime_types::snowbridge_core::outbound::ControlOperation::UpdateChannel,
            update_fee: DEFAULT_OPERATION_FEE,
        },
    );
    let update_fee_for_transfer_native = BHRuntimeCall::EthereumControl(
        runtime_types::snowbridge_control::pallet::Call::update_operation_fee {
            operation:
                runtime_types::snowbridge_core::outbound::ControlOperation::TransferNativeFromAgent,
            update_fee: DEFAULT_OPERATION_FEE,
        },
    );
    let calls = vec![
        update_fee_for_create_agent,
        update_fee_for_create_channel,
        update_fee_for_update_channel,
        update_fee_for_transfer_native,
    ];

    let utility_api = utility::calls::TransactionApi;
    let batch_call = utility_api
        .batch_all(calls)
        .encode_call_data(&test_clients.bridge_hub_client.metadata())
        .expect("encoded call");

    let weight = 180000000000;
    let proof_size = 900000;

    let dest = Box::new(VersionedMultiLocation::V3(MultiLocation {
        parents: 0,
        interior: Junctions::X1(Junction::Parachain(BRIDGE_HUB_PARA_ID)),
    }));
    let message = Box::new(VersionedXcm::V3(Xcm(vec![
        Instruction::UnpaidExecution {
            weight_limit: WeightLimit::Limited(Weight {
                ref_time: weight,
                proof_size,
            }),
            check_origin: None,
        },
        Instruction::Transact {
            origin_kind: OriginKind::Superuser,
            require_weight_at_most: Weight {
                ref_time: weight,
                proof_size,
            },
            call: DoubleEncoded {
                encoded: batch_call,
            },
        },
    ])));

    let sudo_api = relaychain::api::sudo::calls::TransactionApi;
    let sudo_call = sudo_api.sudo(RuntimeCall::XcmPallet(Call::send { dest, message }));

    let result = test_clients
        .relaychain_client
        .tx()
        .sign_and_submit_then_watch_default(&sudo_call, &signer)
        .await
        .expect("send through sudo call.")
        .wait_for_finalized_success()
        .await
        .expect("sudo call success");

    println!(
        "Sudo call issued at relaychain block hash {:?}",
        result.block_hash()
    );

    wait_for_bridgehub_event::<FeeUpdated>(&test_clients.bridge_hub_client).await;
}
