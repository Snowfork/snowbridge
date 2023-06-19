use snowbridge_smoketest::contracts::{native_tokens, weth9};

use std::{sync::Arc, time::Duration};

use ethers::{
    core::types::{Address, U256},
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::parse_units,
};

use codec::Encode;

use xcm::v3::{MultiLocation, Junction, Junctions::X1};

// The deployment addresses of the following contracts are stable, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const TOKEN_VAULT_CONTRACT: &str = "0xB8EA8cB425d85536b158d661da1ef0895Bb92F1D";
const NATIVE_TOKENS_CONTRACT: &str = "0x8cF6147918A5CBb672703F879f385036f8793a24";
const WETH_CONTRACT: &str = "0x440eDFFA1352B13227e8eE646f3Ea37456deC701";

#[tokio::test]
async fn test_lock_tokens() {
    let provider = Provider::<Http>::try_from(ETHEREUM_API)
        .unwrap()
        .interval(Duration::from_millis(10u64));

    let wallet: LocalWallet = ETHEREUM_KEY
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(15u64);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(client);

    let native_tokens_addr = NATIVE_TOKENS_CONTRACT.parse::<Address>().unwrap();
    let native_tokens = native_tokens::NativeTokens::new(native_tokens_addr, client.clone());

    let token_vault_addr = TOKEN_VAULT_CONTRACT.parse::<Address>().unwrap();

    let weth_addr = WETH_CONTRACT.parse::<Address>().unwrap();
    let weth = weth9::WETH9::new(weth_addr, client.clone());

    // Mint WETH tokens
    let value = parse_units("10", "ether").unwrap();
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
    weth.approve(token_vault_addr, value.into())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

    // Ferdie 
    // DE14BzQ1bDXWPKeLoAqdLAm1GpyAWaWF1knF74cEZeomTBM
    // 0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c
    let recipient = MultiLocation {
        parents: 0,
        interior: X1(Junction::AccountId32{
            network: None,
            id: [0x1c, 0xbd, 0x2d, 0x43, 0x53, 0x0a, 0x44, 0x70, 0x5a, 0xd0, 0x88, 0xaf, 0x31, 0x3e, 0x18, 0xf8, 0x0b, 0x53, 0xef, 0x16, 0xb3, 0x61, 0x77, 0xcd, 0x4b, 0x77, 0xb8, 0x46, 0xf2, 0xa5, 0xf0, 0x7c],
        })
    }.encode();

    // Lock tokens into vault
    let value1: u128 = U256::from(value).low_u128();
    native_tokens
        .lock(weth.address(), 0, recipient.into(), value1)
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
