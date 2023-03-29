#![cfg(test)]

mod parachains;
mod contracts;

use parachains::bridgehub;
use contracts::{
    native_tokens::NativeTokens,
    weth9::WETH9,
};


use std::{sync::Arc, time::Duration};

use ethers::{
    middleware::SignerMiddleware,
    signers::LocalWallet,
    signers::Signer,
    providers::{Provider, Http},
    core::{
        types::{Address, U256},
    },
};

#[tokio::test]
async fn test_native_tokens_lock() -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        Provider::<Http>::try_from("http://localhost:8545")?.interval(Duration::from_millis(10u64));

    let wallet: LocalWallet = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342".parse::<LocalWallet>()?.with_chain_id(15u64);

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(client);

    let native_tokens = NativeTokens::new(Address::zero(), client.clone());
    let test_token_address = "0xF8F7758FbcEfd546eAEff7dE24AFf666B6228e73".parse::<Address>()?;
    let test_token = WETH9::new(test_token_address, client.clone());

    let call = test_token.deposit();
    let receipt = call.value(50).send().await?.await?;

    println!("Transaction Receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}
