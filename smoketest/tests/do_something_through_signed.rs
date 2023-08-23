use codec::Encode;
use ethers::prelude::U256;
use ethers::{core::types::Address, utils::parse_units};
use snowbridge_smoketest::constants::GATEWAY_PROXY_CONTRACT;
use snowbridge_smoketest::helper::{initial_clients, wait_for_substrate_event};
use snowbridge_smoketest::parachains::template;
use snowbridge_smoketest::{
    contracts::i_gateway, parachains::template::api::template_pallet::events::SomethingStored,
};
use sp_core::blake2_256;
use subxt::tx::TxPayload;
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

#[tokio::test]
async fn do_something_through_signed() {
    let test_clients = initial_clients().await.expect("initialize clients");

    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let eth_client = *test_clients.ethereum_signed_client;
    let gateway = i_gateway::IGateway::new(gateway_addr, eth_client.clone());

    let sender = AccountId32::from(
        (b"ethereum", 15u64, eth_client.address().as_fixed_bytes()).using_encoded(blake2_256),
    );
    println!("sender address: {}", sender.to_string());

    let template_client: OnlineClient<PolkadotConfig> = *(test_clients.template_client).clone();

    let call = template::api::template_pallet::calls::TransactionApi
        .do_something(1)
        .encode_call_data(&template_client.metadata())
        .expect("create call");

    let fee = parse_units("0.0002", "ether").unwrap();

    let dynamic_fee = parse_units("100000000000000", "wei").unwrap().into();

    let receipt = gateway
        .transact_through_signed_with_destination_chain_and_payload(
            U256::from(1001),
            call.into(),
            dynamic_fee,
            400_000_000,
            8_000,
        )
        .value(fee)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    println!("receipt: {:#?}", hex::encode(receipt.transaction_hash));

    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

    wait_for_substrate_event::<SomethingStored>(&test_clients.template_client).await;
}
