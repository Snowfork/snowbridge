mod estimator;

use crate::estimator::{clients, decode_assets, estimate_gas, EstimatorError};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location;
use clap::{Parser, Subcommand, ValueEnum};
use codec;
use hex;
use std::process;

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
        /// Asset transfer data (JSON array of asset objects)
        #[arg(long, default_value = "[]")]
        assets: String,
        /// Claimer address (hex string)
        #[arg(long)]
        claimer: String,
        /// Origin address (hex string)
        #[arg(long)]
        origin: String,
        /// The full Ether value supplied to cover the transaction, including execution and relayer fee
        #[arg(long)]
        value: u128,
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
    match estimate(cli).await {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

async fn estimate(cli: Cli) -> Result<String, EstimatorError> {
    let clients = clients().await?;

    let (xcm_hex, assets_json, claimer_hex, origin_hex, value, execution_fee, relayer_fee) = match cli.command {
        Commands::V2SendMessage {
            xcm,
            assets,
            claimer,
            origin,
            value,
            execution_fee,
            relayer_fee,
            ..
        } => (xcm, assets, claimer, origin, value, execution_fee, relayer_fee),
    };

    let xcm_bytes = hex::decode(&xcm_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    let claimer_bytes =
        hex::decode(&claimer_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    let origin_bytes =
        hex::decode(&origin_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    let assets = decode_assets(&assets_json)?;

    let claimer: Location = codec::Decode::decode(&mut &claimer_bytes[..])
        .map_err(|_| EstimatorError::InvalidCommand("Failed to decode claimer".to_string()))?;

    if origin_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(
            "Origin must be 20 bytes (Ethereum address)".to_string(),
        ));
    }
    let mut origin = [0u8; 20];
    origin.copy_from_slice(&origin_bytes);

    let estimation = estimate_gas(
        &clients,
        &xcm_bytes,
        claimer,
        origin,
        value,
        execution_fee,
        relayer_fee,
        &assets,
    )
    .await?;

    serde_json::to_string_pretty(&estimation).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to serialize result to JSON: {}", e))
    })
}
