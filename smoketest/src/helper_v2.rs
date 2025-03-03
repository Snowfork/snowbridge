use crate::{
	constants::*,
	contracts::{i_gateway_v1, i_gateway_v2 as i_gateway},
	helper::{fund_account, initial_clients},
};
use ethers::{
	abi::{encode, Token},
	prelude::{Address, EthEvent, Middleware, Provider, Ws},
	types::{Bytes, H160, U256},
};
use futures::StreamExt;
use std::{ops::Deref, sync::Arc};

pub fn build_native_asset(token: H160, amount: u128) -> Bytes {
	let kind_token = Token::Uint(U256::from(0u8));
	let token_token = Token::Address(token);
	let amount_token = Token::Uint(U256::from(amount));

	encode(&[kind_token, token_token, amount_token]).into()
}

pub async fn wait_for_ethereum_event_v2<Ev: EthEvent>(ethereum_client: &Box<Arc<Provider<Ws>>>) {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, (*ethereum_client).deref().clone());

	let wait_for_blocks = 500;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut ethereum_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for expected event", block.number.unwrap());
		if let Ok(events) = gateway.event::<Ev>().at_block_hash(block.hash.unwrap()).query().await {
			for _ in events {
				println!("Event found at ethereum block {:?}", block.number.unwrap());
				ethereum_event_found = true;
				break
			}
		}
		if ethereum_event_found {
			break
		}
	}
	assert!(ethereum_event_found);
}

pub async fn fund_agent_v2(
	agent_id: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");
	let agent_address = get_agent_address(&test_clients.ethereum_client, agent_id)
		.await
		.expect("get agent address");

	fund_account(&test_clients.ethereum_signed_client, agent_address, amount)
		.await
		.expect("fund account");
	Ok(())
}

pub async fn get_agent_address(
	ethereum_client: &Box<Arc<Provider<Ws>>>,
	agent_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, (*ethereum_client).deref().clone());
	let agent_address = gateway.agent_of(agent_id).await.expect("find agent");
	Ok(agent_address)
}

pub async fn get_token_address(
	ethereum_client: &Box<Arc<Provider<Ws>>>,
	token_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway_v1::IGatewayV1::new(gateway_addr, (*ethereum_client).deref().clone());
	let token_address = gateway.token_address_of(token_id).await.expect("find token");
	Ok(token_address)
}
