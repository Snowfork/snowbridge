use ethers::core::types::Address;
use snowbridge_smoketest::{constants::*, contracts::i_gateway_v2 as i_gateway, helper::*};

#[tokio::test]
async fn register_agent() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());
	gateway
		.v2_createAgent(ASSET_HUB_BOB_AGENT_ID)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
}
