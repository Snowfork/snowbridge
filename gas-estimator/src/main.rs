use alloy_sol_types::{sol, SolValue};
use clap::{Parser, Subcommand};
use codec;
use hex;
use snowbridge_gas_estimator::estimator::{
    clients, decode_assets_from_hex, estimate_gas, BridgeAsset, EstimatorError,
};
use snowbridge_gas_estimator::runtimes::Location;
use snowbridge_gas_estimator::xcm_builder::construct_register_token_xcm;
use std::process;

#[derive(Parser)]
#[command(name = "snowbridge-gas-estimator")]
#[command(about = "Off-chain gas estimator for Ethereum -> Polkadot messages via Snowbridge")]
#[command(version = "0.1.0")]
struct Cli {
    /// Asset Hub WebSocket URL
    #[arg(long, required = true)]
    asset_hub_url: String,

    /// Bridge Hub WebSocket URL
    #[arg(long, required = true)]
    bridge_hub_url: String,

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
    /// Estimate gas for submitting a message via BridgeHub
    Message {
        /// Event log address (hex string)
        #[arg(long)]
        event_log_address: String,
        /// Event log topics (comma-separated hex strings)
        #[arg(long)]
        event_log_topics: String,
        /// Event log data (hex string)
        #[arg(long)]
        event_log_data: String,
        /// Proof data (hex-encoded SCALE-encoded proof)
        #[arg(long)]
        proof: String,
        /// XCM kind (0 = Raw, 1 = CreateAsset)
        #[arg(long)]
        xcm_kind: u8,
        /// XCM data (hex string)
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
        /// The full Ether value supplied to cover the transaction
        #[arg(long)]
        value: u128,
        /// Execution fee in wei
        #[arg(long)]
        execution_fee: u128,
        /// Relayer fee in wei
        #[arg(long)]
        relayer_fee: u128,
        /// Relayer account public key (hex string, 32 bytes)
        #[arg(long)]
        relayer_account: String,
        /// Message nonce from the event
        #[arg(long)]
        nonce: u64,
    },
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
    let clients = clients(cli.asset_hub_url, cli.bridge_hub_url).await?;

    match cli.command {
        Commands::Estimate { command } => match command {
            EstimateCommands::Message {
                event_log_address,
                event_log_topics,
                event_log_data,
                proof,
                xcm_kind,
                xcm_data,
                assets,
                claimer,
                origin,
                value,
                execution_fee,
                relayer_fee,
                relayer_account,
                nonce,
            } => {
                let claimer = parse_claimer(&claimer)?;
                let origin = parse_origin(&origin)?;
                let assets = parse_assets(&assets)?;
                let relayer_account = parse_relayer_account(&relayer_account)?;

                // Process XCM based on kind (this is for delivery fee calculation)
                let xcm_bytes = if xcm_kind == 1 {
                    // CreateAsset: xcm_data is ABI-encoded AsCreateAsset{token, network}
                    let create_asset_data = parse_hex_address(&xcm_data)?;
                    let decoded = decode_create_asset(&create_asset_data)?;
                    construct_register_token_xcm(
                        &decoded.token,
                        decoded.network,
                        value,
                        claimer.clone(),
                    )?
                } else {
                    // Raw: xcm_data is SCALE-encoded VersionedXcm
                    parse_hex_address(&xcm_data)?
                };

                let estimation = estimate_gas(
                    &clients,
                    &event_log_address,
                    &event_log_topics,
                    &event_log_data,
                    &proof,
                    &xcm_bytes,
                    claimer,
                    origin,
                    value,
                    execution_fee,
                    relayer_fee,
                    &assets,
                    relayer_account,
                    nonce,
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

fn parse_relayer_account(relayer_account_hex: &str) -> Result<[u8; 32], EstimatorError> {
    let account_bytes = parse_hex_address(relayer_account_hex)?;
    if account_bytes.len() != 32 {
        return Err(EstimatorError::InvalidCommand(
            "Relayer account must be 32 bytes (SR25519 public key)".to_string(),
        ));
    }
    let mut account = [0u8; 32];
    account.copy_from_slice(&account_bytes);
    Ok(account)
}

sol! {
    struct AsCreateAsset {
        address token;
        uint8 network;
    }
}

struct CreateAssetData {
    token: String,
    network: u8,
}

fn decode_create_asset(data: &[u8]) -> Result<CreateAssetData, EstimatorError> {
    let decoded = AsCreateAsset::abi_decode(data).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to decode CreateAsset data: {}", e))
    })?;

    Ok(CreateAssetData {
        token: format!("0x{:x}", decoded.token),
        network: decoded.network,
    })
}
