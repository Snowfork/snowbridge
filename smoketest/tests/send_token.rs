use hex_literal::hex;
use snowbridge_smoketest::contracts::{i_gateway, weth9};
use std::{sync::Arc, time::Duration};

use ethers::{
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::parse_units,
};

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const GATEWAY_PROXY_CONTRACT: &str = "0xEDa338E4dC46038493b885327842fD3E301CaB39";
const WETH_CONTRACT: &str = "0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d";

// SS58: DE14BzQ1bDXWPKeLoAqdLAm1GpyAWaWF1knF74cEZeomTBM
const FERDIE: [u8; 32] = hex!("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c");

// TODO: test sendNativeToken
#[tokio::test]
async fn send_tokens() {
    let provider = Provider::<Http>::try_from(ETHEREUM_API)
        .unwrap()
        .interval(Duration::from_millis(10u64));

    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(15u64);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(client);

    let gateway_addr = GATEWAY_PROXY_CONTRACT.parse::<Address>().unwrap();
    let gateway = i_gateway::IGateway::new(gateway_addr, client.clone());

    let weth_addr = WETH_CONTRACT.parse::<Address>().unwrap();
    let weth = weth9::WETH9::new(weth_addr, client.clone());

    // Mint WETH tokens
    let value = parse_units("1", "ether").unwrap();
    let receipt = weth
        .deposit()
        .value(value)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

    // Approve token spend
    weth.approve(gateway_addr, value.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

    // Lock tokens into vault
    let value1: u128 = U256::from(value).low_u128();
    let receipt = gateway
        .send_token(weth.address(), 1000.into(), FERDIE, value1)
        .value(1000)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    println!("receipt: {:#?}", receipt);

    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);
}
