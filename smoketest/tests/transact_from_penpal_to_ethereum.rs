use ethers::{
	abi::{Abi, Token},
	prelude::{Address, Middleware, Provider, Ws},
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	contracts::hello_world::{HelloWorld, SaidHelloFilter},
	helper::*,
	parachains::penpal::api as PenpalApi,
};
use std::{ops::Deref, sync::Arc};
use subxt::{
	ext::sp_core::{sr25519::Pair, Pair as PairT},
	tx::PairSigner,
};

const HELLO_WORLD_CONTRACT: [u8; 20] = hex!("EE9170ABFbf9421Ad6DD07F6BDec9D89F2B581E0");

#[tokio::test]
async fn transact_from_penpal_to_ethereum() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let ethereum_client = *(test_clients.ethereum_client.clone());
	let penpal_client = *(test_clients.penpal_client.clone());

	let hello_world = HelloWorld::new(HELLO_WORLD_CONTRACT, ethereum_client.clone());
	let contract_abi: Abi = hello_world.abi().clone();
	let function = contract_abi.function("sayHello").unwrap();
	let encoded_data =
		function.encode_input(&[Token::String("Hello, Clara!".to_string())]).unwrap();

	println!("data is {}", hex::encode(encoded_data.clone()));

	let extrinsic_call = PenpalApi::transact_helper::calls::TransactionApi.transact_to_ethereum(
		HELLO_WORLD_CONTRACT.into(),
		encoded_data,
		4_000_000_000,
		80_000,
	);

	let owner: Pair = Pair::from_string("//Bob", None).expect("cannot create keypair");
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new(owner);

	let _ = penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&extrinsic_call, &signer)
		.await
		.expect("send through xcm call.");

	wait_for_arbitrary_transact_event(&test_clients.ethereum_client, HELLO_WORLD_CONTRACT).await;
}

pub async fn wait_for_arbitrary_transact_event(
	ethereum_client: &Box<Arc<Provider<Ws>>>,
	contract_address: [u8; 20],
) {
	let addr: Address = contract_address.into();
	let contract = HelloWorld::new(addr, (*ethereum_client).deref().clone());

	let wait_for_blocks = 300;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut ethereum_event_found = false;
	while let Some(block) = stream.next().await {
		if let Ok(events) = contract
			.event::<SaidHelloFilter>()
			.at_block_hash(block.hash.unwrap())
			.query()
			.await
		{
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
