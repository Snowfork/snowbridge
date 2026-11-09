//! Live probe for successful V2 nonce 604 / message 0x711d...62cc.
//!
//! Decoded from https://etherscan.io/tx/0x5f7f4e9da43e6f49e9fbcfe5c5b699776e8fe458510af7fe031cd33a709f9c58
//!
//! Usage:
//!   cargo run --example dry_run_ah_succeed_604 --features polkadot -- \
//!     --asset-hub-url wss://polkadot-asset-hub-rpc.polkadot.io \
//!     --bridge-hub-url wss://polkadot-bridge-hub-rpc.polkadot.io

use clap::Parser;
use codec::Decode;
use snowbridge_gas_estimator::config::BRIDGE_HUB_PARA_ID;
use snowbridge_gas_estimator::estimator::clients;
use snowbridge_gas_estimator::runtimes::{
    asset_hub_runtime, Junction, Junctions::X1, Location, Outcome, VersionedLocation,
};
use snowbridge_gas_estimator::xcm_builder::{build_asset_hub_xcm, get_claimer_location};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    asset_hub_url: String,
    #[arg(long)]
    bridge_hub_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gateway OutboundMessageAccepted nonce 604
    // message id 0x711d049d77bcefb8b6402fa0ba0b13c859f44e29aefc7670395765f4678e62cc
    let origin: [u8; 20] = hex::decode("091229c0b1465dd8d9977f1c45d368bbc34b8405")?
        .try_into()
        .map_err(|_| "bad origin")?;
    let value: u128 = 1_000_452_951_065_380;
    let execution_fee: u128 = 16_645_917_199_854;
    let xcm_hex = "051031010100c91f010100040201090704000724c3fc756900040100040201090704000f0080c6a47e8d0310140d01010201090704000001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a0d0102040001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2c711d049d77bcefb8b6402fa0ba0b13c859f44e29aefc7670395765f4678e62cc140d01010201090704000001010054d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a2c711d049d77bcefb8b6402fa0ba0b13c859f44e29aefc7670395765f4678e62cc";
    let claimer_hex = "000101010254d82b42bcd22b175d71d62ef2114defcf14344c4b88acf0eb4356737d7fdb4a";

    let args = Args::parse();
    println!("connecting…");
    let clients = clients(args.asset_hub_url, args.bridge_hub_url).await?;

    let claimer_bytes = hex::decode(claimer_hex)?;
    let claimer = match Location::decode(&mut &claimer_bytes[..]) {
        Ok(loc) => Some(loc),
        Err(e) => {
            eprintln!("claimer decode failed ({e:?}); falling back to bridge owner");
            None
        }
    };
    let claimer_location = get_claimer_location(claimer)?;

    let xcm_bytes = hex::decode(xcm_hex)?;
    println!("building destination_xcm…");
    let destination_xcm = build_asset_hub_xcm(
        &clients,
        &xcm_bytes,
        claimer_location,
        origin,
        value,
        execution_fee,
        &[], // no ERC-20 assets in this transfer
    )
    .await?;

    let origin_location = VersionedLocation::V5(Location {
        parents: 1,
        interior: X1([Junction::Parachain(BRIDGE_HUB_PARA_ID)]),
    });

    println!("calling AssetHub DryRunApi::dry_run_xcm…");
    let runtime_api_call = asset_hub_runtime::apis()
        .dry_run_api()
        .dry_run_xcm(origin_location, destination_xcm.clone());

    let result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;

    match result {
        Ok(effects) => {
            let success = matches!(effects.execution_result, Outcome::Complete { .. });
            println!("dry_run_xcm success = {success}");
            println!("execution_result = {:?}", effects.execution_result);
            println!(
                "emitted_events = {} forwarded_xcms = {}",
                effects.emitted_events.len(),
                effects.forwarded_xcms.len()
            );
            if !success {
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("DryRunApi error: {e:?}");
            std::process::exit(1);
        }
    }

    // Keep destination_xcm in scope for debug if needed
    let _ = destination_xcm;
    Ok(())
}
