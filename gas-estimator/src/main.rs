mod estimator;

use crate::estimator::{EstimatorError, clients, estimate_gas};
use clap::{Parser, Subcommand, ValueEnum};
use std::process;
use hex;
use codec;
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

async fn estimate(
    cli: Cli
) -> Result<String, EstimatorError> {
    let clients = clients().await?;

    let (xcm_hex, claimer_hex) = match cli.command {
        Commands::V2SendMessage { xcm, claimer, .. } => (xcm, claimer),
    };

    let xcm_bytes = hex::decode(&xcm_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    let claimer_bytes = hex::decode(&claimer_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;

    let claimer: Location = codec::Decode::decode(&mut &claimer_bytes[..])
        .map_err(|_| EstimatorError::InvalidCommand("Failed to decode claimer".to_string()))?;

    estimate_gas(&clients, &xcm_bytes, claimer).await
}
