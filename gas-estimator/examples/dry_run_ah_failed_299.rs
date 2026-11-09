//! Live probe for failed V2 nonce 299 (TooExpensive on Asset Hub).
//!
//! Message ID: `0x2fef7fdfd9fa5b02e398d090237630bad377db6617b1d0f94575933d9f6cc280`
//!
//! Original tx trail (2026-05-13):
//! - Optimism L2 source: `0xa815ad107e0922daa463fbb0ba0ce18263f16f4519394a15ec0a2a35a987afdd`
//! - Bridge Hub submit:  `0x8575053619b28a01b22c5b33b20dc93222f6ed7beb77adca7da803bfbf5212d3`
//!   (extrinsic in BH block 7649824; EventProof recovered from this extrinsic)
//! - Asset Hub failure:  block 15707436 — `PolkadotXcm.ProcessXcmError(TooExpensive)`
//!   then `MessageQueue.Processed(..., false)`
//!
//! Usage:
//!   cargo run --example dry_run_ah_failed_299 --features polkadot -- \
//!     --asset-hub-url wss://polkadot-asset-hub-rpc.polkadot.io \
//!     --bridge-hub-url wss://polkadot-bridge-hub-rpc.polkadot.io
//!
//! To reproduce TooExpensive exactly, dry-run at AH parent of 15707436:
//!   --at 0x350dd828c03de6ea3a39adb014a1580c2c84ff4cfd6f63470d6424d370826a23

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
    /// Optional Asset Hub block hash to dry-run at (default: latest).
    #[arg(long)]
    at: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gateway OutboundMessageAccepted nonce 299 / message 0x2fef7fdf...c280
    // (decoded from Bridge Hub submit extrinsic EventProof above)
    let origin: [u8; 20] = hex::decode("924a9f036260ddd5808007e1aa95f08ed08aa569")?
        .try_into()
        .map_err(|_| "bad origin")?;
    let value: u128 = 1_000_000_000_000_000; // 0.001 ETH
    let execution_fee: u128 = 1_627_494_698_874; // ~1.6e-6 ETH — historically TooExpensive
    let xcm_hex = "051414180802010907040007a836e92cb802020907040300a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4800025a620200000d010208000101002134cfef589b0b22614d0316ba90586dc6909ea70000000000000000000000000d010208000101002134cfef589b0b22614d0316ba90586dc6909ea70000000000000000000000002c2fef7fdfd9fa5b02e398d090237630bad377db6617b1d0f94575933d9f6cc280";
    let claimer_hex = "00010101022134cfef589b0b22614d0316ba90586dc6909ea7000000000000000000000000";

    let args = Args::parse();
    println!("connecting…");
    let clients = clients(args.asset_hub_url, args.bridge_hub_url).await?;

    let claimer_bytes = hex::decode(claimer_hex)?;
    let claimer = Location::decode(&mut &claimer_bytes[..]).ok();
    let claimer_location = get_claimer_location(claimer)?;

    let xcm_bytes = hex::decode(xcm_hex)?;
    println!(
        "building destination_xcm for nonce 299 (execution_fee={execution_fee} wei)…"
    );
    let destination_xcm = build_asset_hub_xcm(
        &clients,
        &xcm_bytes,
        claimer_location,
        origin,
        value,
        execution_fee,
        &[],
    )
    .await?;

    let origin_location = VersionedLocation::V5(Location {
        parents: 1,
        interior: X1([Junction::Parachain(BRIDGE_HUB_PARA_ID)]),
    });

    println!("calling AssetHub DryRunApi::dry_run_xcm…");
    let runtime_api_call = asset_hub_runtime::apis()
        .dry_run_api()
        .dry_run_xcm(origin_location, destination_xcm);

    let api = clients.asset_hub_client.runtime_api();
    let result = if let Some(at) = args.at.as_deref() {
        let hash_bytes = hex::decode(at.trim_start_matches("0x"))?;
        let hash = subxt::utils::H256::from_slice(&hash_bytes);
        println!("at block {at}");
        api.at(hash).call(runtime_api_call).await?
    } else {
        api.at_latest().await?.call(runtime_api_call).await?
    };

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
            // Print any ProcessXcmError / fee-related hints from Debug of outcome
            if success {
                eprintln!("UNEXPECTED: historically this message failed with TooExpensive");
                std::process::exit(3);
            } else {
                println!("FAILED as expected for underfunded execution fee path");
            }
        }
        Err(e) => {
            eprintln!("DryRunApi error: {e:?}");
            std::process::exit(1);
        }
    }
    Ok(())
}
