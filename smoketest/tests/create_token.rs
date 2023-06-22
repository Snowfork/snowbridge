use snowbridge_smoketest::contracts::{native_tokens, weth9};

use std::{sync::Arc, time::Duration};

use ethers::{
    core::types::Address,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
};

// The deployment addresses of the following contracts are stable, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const NATIVE_TOKENS_CONTRACT: &str = "0x8cF6147918A5CBb672703F879f385036f8793a24";
const WETH_CONTRACT: &str = "0x440eDFFA1352B13227e8eE646f3Ea37456deC701";

#[tokio::test]
async fn create_tokens() {
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

    let weth_addr = WETH_CONTRACT.parse::<Address>().unwrap();
    let weth = weth9::WETH9::new(weth_addr, client.clone());

    let receipt = native_tokens
        .create(weth.address())
        .value(1000)
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.status.unwrap().as_u64(), 1u64);
}
