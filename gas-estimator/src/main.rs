mod estimator;
mod penpal;

use crate::estimator::{
    clients, construct_register_token_xcm, decode_assets, estimate_gas, BridgeAsset, EstimatorError,
};
#[cfg(feature = "local")]
use asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::location::Location;
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
    /// Estimate gas for Snowbridge operations
    Estimate {
        #[command(subcommand)]
        command: EstimateCommands,
    },
}

#[derive(Subcommand)]
enum EstimateCommands {
    /// Estimate gas for sending a message
    SendMessage {
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
    /// Estimate gas for registering a token
    RegisterToken {
        /// Token address (hex string)
        #[arg(long)]
        token_address: String,
        /// Network ID (currently only 0 is supported)
        #[arg(long, default_value = "0")]
        network: u8,
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

    match cli.command {
        Commands::Estimate { command } => match command {
            EstimateCommands::SendMessage {
                xcm,
                assets,
                claimer,
                origin,
                value,
                execution_fee,
                relayer_fee,
            } => {
                // Parse common parameters
                let xcm_bytes = parse_hex_address(&xcm, "xcm")?;
                let claimer = parse_claimer(&claimer)?;
                let origin = parse_origin(&origin)?;
                let assets = parse_assets(&assets)?;

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
                    EstimatorError::InvalidCommand(format!(
                        "Failed to serialize result to JSON: {}",
                        e
                    ))
                })
            }
            EstimateCommands::RegisterToken {
                token_address,
                network,
                assets,
                claimer,
                origin,
                value,
                execution_fee,
                relayer_fee,
            } => {
                // Validate network
                if network != 0 {
                    return Err(EstimatorError::InvalidCommand(format!(
                        "Unsupported network: {}. Currently only network 0 is supported",
                        network
                    )));
                }

                // Parse common parameters
                let claimer = parse_claimer(&claimer)?;
                let origin = parse_origin(&origin)?;
                let assets = parse_assets(&assets)?;

                // Construct register token XCM
                let xcm_bytes =
                    construct_register_token_xcm(&token_address, network, value, claimer.clone())?;

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
                    EstimatorError::InvalidCommand(format!(
                        "Failed to serialize result to JSON: {}",
                        e
                    ))
                })
            }
        },
    }
}

fn parse_hex_address(hex_str: &str, name: &str) -> Result<Vec<u8>, EstimatorError> {
    hex::decode(&hex_str[2..]).map_err(|_| EstimatorError::InvalidHexFormat)
}

fn parse_claimer(claimer_hex: &str) -> Result<Location, EstimatorError> {
    let claimer_bytes = parse_hex_address(claimer_hex, "claimer")?;
    codec::Decode::decode(&mut &claimer_bytes[..])
        .map_err(|_| EstimatorError::InvalidCommand("Failed to decode claimer".to_string()))
}

fn parse_origin(origin_hex: &str) -> Result<[u8; 20], EstimatorError> {
    let origin_bytes = parse_hex_address(origin_hex, "origin")?;
    if origin_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(
            "Origin must be 20 bytes (Ethereum address)".to_string(),
        ));
    }
    let mut origin = [0u8; 20];
    origin.copy_from_slice(&origin_bytes);
    Ok(origin)
}

fn parse_assets(assets_json: &str) -> Result<Vec<BridgeAsset>, EstimatorError> {
    decode_assets(assets_json)
}

fn parse_token_address(token_address_hex: &str) -> Result<[u8; 20], EstimatorError> {
    let token_bytes = parse_hex_address(token_address_hex, "token_address")?;
    if token_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(
            "Token address must be 20 bytes (H160)".to_string(),
        ));
    }
    let mut token_address = [0u8; 20];
    token_address.copy_from_slice(&token_bytes);
    Ok(token_address)
}
