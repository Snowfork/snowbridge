use ethers::prelude::Address;
use snowbridge_smoketest::constants::*;
use snowbridge_smoketest::contracts::i_gateway;
use snowbridge_smoketest::contracts::i_gateway::InboundMessageDispatchedFilter;
use snowbridge_smoketest::helper::*;
use snowbridge_smoketest::parachains::bridgehub::api::ethereum_control::events::TransferNativeFromAgent;
use snowbridge_smoketest::xcm::construct_xcm_message;

#[tokio::test]
async fn transact() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let ethereum_client = *(test_clients.ethereum_client.clone());
    let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());
    let agent_address = gateway
        .agent_of(SIBLING_AGENT_ID)
        .await
        .expect("find agent");

    println!("agent address {}", hex::encode(agent_address));

    fund_account(&test_clients.ethereum_signed_client, agent_address)
        .await
        .expect("fund account");

    let message = construct_xcm_message(
        construct_transact_call(
            &test_clients.bridge_hub_client,
            ETHEREUM_ADDRESS.into(), // TODO TARGET ARBITRARY CONTRACT
            vec![], // TODO CONTRACT BYTES HERE
        )
            .await
            .expect("construct inner call."),
    );

    let result = send_xcm_transact(&test_clients.template_client, message)
        .await
        .expect("failed to send xcm transact.");

    println!(
        "xcm call issued at block hash {:?}, transaction hash {:?}",
        result.block_hash(),
        result.extrinsic_hash()
    );

    //wait_for_bridgehub_event::<AgentExecute>(&test_clients.bridge_hub_client).await;
}
