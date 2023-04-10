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

use xcm::v3::MultiLocation;

// The deployment addresses of the following contracts are stable, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const NATIVE_TOKENS_CONTRACT: &str = "0x83428c7db9815f482a39a1715684dCF755021997";
const TOKEN_VAULT_CONTRACT: &str = "0x774667629726ec1FaBEbCEc0D9139bD1C8f72a23";
const WETH_CONTRACT: &str = "0xF8F7758FbcEfd546eAEff7dE24AFf666B6228e73";

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

    let recipient = MultiLocation::default().encode();

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
