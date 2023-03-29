mod parachains;
mod contracts;

use parachains::bridgehub;
use contracts::native_tokens;

use std::{sync::Arc, time::Duration};

use ethers::{
    middleware::SignerMiddleware,
    signers::LocalWallet,
    providers::{Provider, Http},
    core::{
        types::{Address, U256},
    }
};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// Name of the person to greet
	#[clap(short, long)]
	ethereum_key: String,

	/// Number of times to greet
	#[clap(short, long)]
	substrate_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();

    let provider =
        Provider::<Http>::try_from(&args.ethereum_key)?.interval(Duration::from_millis(10u64));

    let wallet: LocalWallet = "YOUR PRIVATE KEY"
        .parse::<LocalWallet>()?;

    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let client = Arc::new(client);

    let native_tokens = native_tokens::NativeTokens::new(Address::zero(), client);
    let test_token = 

    contract.lock(token, recipient, amount);

    Ok(())
}
