mod config;
mod contracts;
mod estimator;

use crate::estimator::{
    clients, construct_register_token_xcm, decode_assets_from_hex, estimate_gas, BridgeAsset,
    EstimatorError,
};
use alloy_sol_types::{sol, SolValue};
#[cfg(feature = "local")]
use asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::location::Location;
use clap::{Parser, Subcommand, ValueEnum};
use codec;
use hex;
use std::process;

// Define the AsCreateAsset struct from the Solidity contract
sol! {
    struct AsCreateAsset {
        address token;
        uint8 network;
    }
}

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
    /// Estimate gas for a message (handles both raw XCM and create asset based on XCM kind)
    Message {
        /// XCM kind (0 = Raw, 1 = CreateAsset)
        #[arg(long)]
        xcm_kind: u8,
        /// XCM data (hex string) - raw XCM bytes for kind=0, ABI-encoded AsCreateAsset for kind=1
        #[arg(long)]
        xcm_data: String,
        /// Asset transfer data (hex-encoded asset data)
        #[arg(long, default_value = "")]
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
            EstimateCommands::Message {
                xcm_kind,
                xcm_data,
                assets,
                claimer,
                origin,
                value,
                execution_fee,
                relayer_fee,
            } => {
                let claimer = parse_claimer(&claimer)?;
                let origin = parse_origin(&origin)?;
                let assets = parse_assets(&assets)?;

                // Process XCM based on kind
                let xcm_bytes = match xcm_kind {
                    0 => {
                        // Raw XCM bytes
                        parse_hex_address(&xcm_data)?
                    }
                    1 => {
                        // CreateAsset - decode token address and network from ABI-encoded data
                        let create_asset_data = parse_hex_address(&xcm_data)?;
                        let (token_address, network) =
                            decode_create_asset_data(&create_asset_data)?;
                        construct_register_token_xcm(
                            &format!("0x{}", hex::encode(token_address)),
                            network,
                            value,
                            claimer.clone(),
                        )?
                    }
                    _ => {
                        return Err(EstimatorError::InvalidCommand(format!(
                            "Unsupported XCM kind: {}. Must be 0 (Raw) or 1 (CreateAsset)",
                            xcm_kind
                        )));
                    }
                };

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

fn parse_hex_address(hex_str: &str) -> Result<Vec<u8>, EstimatorError> {
    if hex_str.len() < 2 {
        return Ok(vec![]);
    }
    hex::decode(&hex_str[2..]).map_err(|_| EstimatorError::InvalidHexFormat)
}

fn parse_claimer(claimer_hex: &str) -> Result<Option<Location>, EstimatorError> {
    let claimer_bytes = parse_hex_address(claimer_hex)?;
    if claimer_bytes.len() < 2 {
        return Ok(None);
    }
    let location = codec::Decode::decode(&mut &claimer_bytes[..])
        .map_err(|_| EstimatorError::InvalidCommand("Failed to decode claimer".to_string()))?;
    Ok(Some(location))
}

fn parse_origin(origin_hex: &str) -> Result<[u8; 20], EstimatorError> {
    let origin_bytes = parse_hex_address(origin_hex)?;
    if origin_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(
            "Origin must be 20 bytes (Ethereum address)".to_string(),
        ));
    }
    let mut origin = [0u8; 20];
    origin.copy_from_slice(&origin_bytes);
    Ok(origin)
}

fn parse_assets(assets_hex: &str) -> Result<Vec<BridgeAsset>, EstimatorError> {
    decode_assets_from_hex(assets_hex)
}

fn decode_create_asset_data(data: &[u8]) -> Result<([u8; 20], u8), EstimatorError> {
    // Decode the ABI-encoded AsCreateAsset struct using alloy
    let decoded = AsCreateAsset::abi_decode(data).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to decode AsCreateAsset: {}", e))
    })?;

    // Convert alloy Address to [u8; 20]
    let token_address: [u8; 20] = decoded.token.into();

    Ok((token_address, decoded.network))
}
