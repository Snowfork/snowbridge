use snowbridge_smoketest::helper::{
    governance_bridgehub_call_from_relay_chain, initial_clients, wait_for_bridgehub_event,
};
use snowbridge_smoketest::parachains::bridgehub::api::ethereum_outbound_queue::events::BaseFeeSet;
use snowbridge_smoketest::parachains::bridgehub::{
    api::runtime_types, api::runtime_types::bridge_hub_rococo_runtime::RuntimeCall as BHRuntimeCall,
};
use std::env;

#[tokio::test]
async fn configure_base_fee() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let default_operation_fee: u128 = env::var("BASE_FEE")
        .unwrap_or("100000000000".parse().unwrap())
        .parse::<u128>()
        .unwrap();

    let update_base_fee = BHRuntimeCall::EthereumOutboundQueue(
        runtime_types::snowbridge_outbound_queue::pallet::Call::set_base_fee {
            amount: default_operation_fee,
        },
    );
    let calls = vec![update_base_fee];

    governance_bridgehub_call_from_relay_chain(calls)
        .await
        .expect("governance call from relaychain by xcm");

    wait_for_bridgehub_event::<BaseFeeSet>(&test_clients.bridge_hub_client).await;
}
