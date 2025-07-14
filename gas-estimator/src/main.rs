mod estimator;

use crate::estimator::{EstimatorError, clients, build_asset_hub_xcm};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json;
use std::process;
use hex;
use codec;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedXcm;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location;

#[derive(Parser)]
#[command(name = "snowbridge-gas-estimator")]
#[command(about = "Off-chain gas estimator for Ethereum -> Polkadot messages via Snowbridge")]
#[command(version = "0.1.0")]
struct Cli {
    /// Environment
    #[arg(long, value_enum, default_value_t = Environment::PolkadotMainnet)]
    env: Environment,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a message via v2_sendMessage
    V2SendMessage {
        /// XCM payload bytes
        #[arg(long)]
        xcm: String,
        /// Asset transfer data (JSON array of hex strings)
        #[arg(long, default_value = "[]")]
        assets: String,
        /// Claimer address (hex string)
        #[arg(long)]
        claimer: String,
        /// Execution fee in wei
        #[arg(long)]
        execution_fee: u128,
        /// Relayer fee in wei
        #[arg(long)]
        relayer_fee: u128,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Environment {
    #[value(name = "polkadot_mainnet")]
    PolkadotMainnet,
    #[value(name = "westend_sepolia")]
    WestendSepolia,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Run the estimation
    match run_estimation(cli).await {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

async fn run_estimation(
    cli: Cli
) -> Result<String, EstimatorError> {
    let clients = clients().await?;

    // Extract the fields from the command
    let (xcm_hex, claimer_hex) = match cli.command {
        Commands::V2SendMessage { xcm, claimer, .. } => (xcm, claimer),
    };

    // Convert hex strings to bytes
    let xcm_bytes = hex::decode(&xcm_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    let claimer_bytes = hex::decode(&claimer_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;

    let claimer: Location = codec::Decode::decode(&mut &claimer_bytes[..])
        .map_err(|_| EstimatorError::InvalidCommand("Failed to decode claimer".to_string()))?;

    let destination_xcm = build_asset_hub_xcm(xcm_bytes, claimer);

    let runtime_api_call = asset_hub_westend_runtime::runtime::apis().xcm_payment_api().query_xcm_weight(destination_xcm);

    let weight_result = clients.asset_hub_client
        .runtime_api()
        .at_latest()
        .await.map_err(|_| EstimatorError::InvalidHexFormat)?
        .call(runtime_api_call)
        .await;

    Ok(format!("XCM weight query result: {:?}", weight_result))
}

//fn format_output(
//    estimation: &crate::types::GasEstimation,
//    format: OutputFormat,
//) -> Result<String, EstimatorError> {
//    match format {
//        OutputFormat::Json => {
//            serde_json::to_string_pretty(estimation)
//                .map_err(|e| EstimatorError::ConfigError(format!("JSON serialization failed: {}", e)))
//        }
//    }
//}
