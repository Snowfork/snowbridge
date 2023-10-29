use snowbridge_smoketest::contracts::i_gateway::AgentCreatedFilter;
use snowbridge_smoketest::helper::*;
use snowbridge_smoketest::parachains::bridgehub::api::ethereum_control::events::CreateAgent;
use snowbridge_smoketest::xcm::construct_xcm_message_with_fee;

#[tokio::test]
async fn create_agent() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let encoded_call = construct_create_agent_call(&test_clients.bridge_hub_client)
        .await
        .expect("construct inner call.");

    let message = construct_xcm_message_with_fee(encoded_call).await;

    let result = send_xcm_transact(&test_clients.template_client, message)
        .await
        .expect("failed to send xcm transact.");

    println!(
        "xcm call issued at block hash {:?}, transaction hash {:?}",
        result.block_hash(),
        result.extrinsic_hash()
    );

    wait_for_bridgehub_event::<CreateAgent>(&test_clients.bridge_hub_client).await;

    wait_for_ethereum_event::<AgentCreatedFilter>(&test_clients.ethereum_client).await;
}
