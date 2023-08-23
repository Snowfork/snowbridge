use snowbridge_smoketest::contracts::i_gateway::ChannelCreatedFilter;
use snowbridge_smoketest::helper::*;
use snowbridge_smoketest::parachains::bridgehub::api::ethereum_control::events::CreateChannel;
use snowbridge_smoketest::xcm::construct_xcm_message;

#[tokio::test]
async fn create_channel() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let message = construct_xcm_message(
        construct_create_channel_call(&test_clients.bridge_hub_client)
            .await
            .expect("construct innner call."),
    );

    let result = send_xcm_transact(&test_clients.template_client, message)
        .await
        .expect("failed to send xcm transact.");

    println!(
        "xcm call issued at block hash {:?}, transaction hash {:?}",
        result.block_hash(),
        result.extrinsic_hash()
    );

    wait_for_substrate_event::<CreateChannel>(&test_clients.bridge_hub_client).await;

    wait_for_ethereum_event::<ChannelCreatedFilter>(&test_clients.ethereum_client).await;
}
