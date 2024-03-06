use std::{convert::TryFrom, sync::Arc};

use ethers::{
	prelude::*,
	providers::{Http, Provider},
	utils::keccak256,
};
use hex_literal::hex;
use snowbridge_smoketest::contracts::i_gateway;

const URL: &str = "https://sepolia.infura.io/v3/1989f3c5cbc549d9b5c37f64e948fa58";

#[tokio::test]
async fn prepare_upgrade_code_hash() {
	let ethereum_provider = Provider::<Http>::try_from(URL).unwrap();

	let chain_id = ethereum_provider.get_chainid().await.unwrap();
	println!("chain id: {:#?}", chain_id);

	const GATETWAY_LOGIC_CONTRACT: [u8; 20] = hex!("102D4EB3973bE1Ba202F89aD948E854973C33d2D");

	let code = ethereum_provider
		.get_code(NameOrAddress::Address(GATETWAY_LOGIC_CONTRACT.into()), None)
		.await
		.unwrap();

	let gateway_logic_code_hash: H256 = keccak256(code).into();
	println!("code hash: {:#?}", hex::encode(gateway_logic_code_hash));
}

#[tokio::test]
async fn fetch_nonces() {
	let ethereum_provider = Provider::<Http>::try_from(URL).unwrap();
	let ethereum_client = Arc::new(ethereum_provider);

	const GATETWAY_ADDRESS: [u8; 20] = hex!("5b4909ce6ca82d2ce23bd46738953c7959e710cd");
	const ASSET_HUB_CHANNEL_ID: [u8; 32] =
		hex!("c173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539");

	let gateway = i_gateway::IGateway::new(GATETWAY_ADDRESS, ethereum_client.clone());

	let nonces = gateway.channel_nonces_of(ASSET_HUB_CHANNEL_ID).await.unwrap();

	println!("inbound nonce: {:#?}, outbound nonce: {:#?}", nonces.0, nonces.1);
}
