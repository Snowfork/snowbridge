use codec::Encode;
use ethers::prelude::U256;
use ethers::{
    core::types::Address,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::parse_units,
    utils::rlp::Encodable,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::parachains::template;
use snowbridge_smoketest::{
    contracts::i_gateway, parachains::template::api::template_pallet::events::SomethingStored,
};
use sp_core::blake2_256;
use std::{sync::Arc, time::Duration};
use subxt::tx::TxPayload;
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const TEMPLATE_WS_URL: &str = "ws://127.0.0.1:13144";
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const GATEWAY_PROXY_CONTRACT: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");

#[tokio::test]
async fn do_something_as_arbitrary_transact() {
    let provider = Provider::<Http>::try_from(ETHEREUM_API)
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(15u64);
    let client = SignerMiddleware::new(provider, wallet.clone());
    let client = Arc::new(client);

    let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
    let gateway = i_gateway::IGateway::new(gateway_addr, client.clone());

    let sender = AccountId32::from(
        (b"ethereum", 15u64, wallet.address().as_fixed_bytes()).using_encoded(blake2_256),
    );
    println!("sender address: {}", sender.to_string());

    let template_client: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(TEMPLATE_WS_URL).await.unwrap();

    let call = template::api::template_pallet::calls::TransactionApi
        .do_something(1)
        .encode_call_data(&template_client.metadata())
        .expect("create call");

    let fee = parse_units("0.0002", "ether").unwrap();

    let dynamic_fee = parse_units("100000000000000", "wei").unwrap().into();

    let receipt = gateway
        .transact_with_destination_chain_and_payload(
            // todo: temporarily use channel of asset_hub, will change to 1001(template) when PR#917 merged
            // require agent/channel created
            U256::from(1000),
            call.into(),
            dynamic_fee,
            400_000_000,
            8_000,
        )
        // Or just use default
        // .send_transact(U256::from(1000), create_call.into())
        .value(fee)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    println!("receipt: {:#?}", hex::encode(receipt.transaction_hash));

    // Log for OutboundMessageAccepted
    let outbound_message_accepted_log = receipt.logs.last().unwrap();
    // RLP-encode log and print it
    println!(
        "receipt log: {:#?}",
        hex::encode(outbound_message_accepted_log.rlp_bytes())
    );

    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

    let wait_for_blocks = 50;
    let mut blocks = template_client
        .blocks()
        .subscribe_finalized()
        .await
        .expect("block subscription")
        .take(wait_for_blocks);

    let mut event_found = false;
    while let Some(Ok(block)) = blocks.next().await {
        println!("Polling block {} for event.", block.number());

        let events = block.events().await.unwrap();
        for event in events.find::<SomethingStored>() {
            println!("event found in block {}.", block.number());
            event_found = true;
            assert_eq!(event.unwrap().0, 1);
        }
        if event_found {
            break;
        }
    }
    assert!(event_found)
}
