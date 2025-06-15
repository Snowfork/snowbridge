use crate::{
	constants::*,
	contracts::{i_gateway_v1, i_gateway_v2 as i_gateway},
	helper::{fund_account, initial_clients},
};
use alloy::{
	dyn_abi::DynSolValue,
	primitives::{Address, Bytes, FixedBytes, U256},
	providers::DynProvider,
};

pub fn build_native_asset(token: Address, amount: u128) -> Bytes {
	let kind_token = DynSolValue::Uint(U256::from(0u8), 256);
	let token_token = DynSolValue::Address(token);
	let amount_token = DynSolValue::Uint(U256::from(amount), 256);
	Bytes::from(DynSolValue::Tuple(vec![kind_token, token_token, amount_token]).abi_encode())
}

pub async fn fund_agent_v2(
	agent_id: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");
	let agent_address = get_agent_address(test_clients.ethereum_client.clone(), agent_id)
		.await
		.expect("get agent address");

	fund_account(test_clients.ethereum_client, agent_address, amount)
		.await
		.expect("fund account");
	Ok(())
}

pub async fn get_agent_address(
	ethereum_client: Box<DynProvider>,
	agent_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, *ethereum_client);
	let agent_address = gateway.agentOf(FixedBytes::from(agent_id)).call().await?;
	Ok(agent_address)
}

pub async fn get_token_address(
	ethereum_client: Box<DynProvider>,
	token_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway_v1::IGatewayV1::new(gateway_addr, *ethereum_client);
	let token_address = gateway.tokenAddressOf(FixedBytes::from(token_id)).call().await?;
	Ok(token_address)
}
