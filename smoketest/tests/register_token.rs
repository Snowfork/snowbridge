use snowbridge_smoketest::contracts::{i_gateway, weth9};

use std::{sync::Arc, time::Duration};

use ethers::{
    core::types::Address,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::parse_units,
    utils::rlp::Encodable,
};

// The deployment addresses of the following contracts are stable, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const GATEWAY_PROXY_CONTRACT: &str = "0xEDa338E4dC46038493b885327842fD3E301CaB39";
const WETH_CONTRACT: &str = "0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d";

#[tokio::test]
async fn register_token() {
    let provider = Provider::<Http>::try_from(ETHEREUM_API)
        .unwrap()
        .interval(Duration::from_millis(10u64));
    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(15u64);
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    let gateway_addr = GATEWAY_PROXY_CONTRACT.parse::<Address>().unwrap();
    let gateway = i_gateway::IGateway::new(gateway_addr, client.clone());

    let weth_addr = WETH_CONTRACT.parse::<Address>().unwrap();
    let weth = weth9::WETH9::new(weth_addr, client.clone());

    let fee = parse_units(2, "ether").unwrap();

    let receipt = gateway
        .register_token(weth.address())
        .value(fee)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    // Log for OutboundMessageAccepted
    let outbound_message_accepted_log = receipt.logs.last().unwrap();
    // RLP-encode log and print it
    println!(
        "receipt: {:#?}",
        hex::encode(outbound_message_accepted_log.rlp_bytes())
    );

    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);
}
