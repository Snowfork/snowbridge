mod asset_hub_runtime;
mod bridge_hub_runtime;
mod chopsticks;
mod commands;
mod constants;
mod helpers;
mod relay_runtime;

use alloy_primitives::{utils::parse_units, Address, Bytes, FixedBytes, U128, U256};
use chopsticks::generate_chopsticks_script;
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::Encode;
use constants::{ASSET_HUB_API, BRIDGE_HUB_API, POLKADOT_DECIMALS, POLKADOT_SYMBOL, RELAY_API};
use helpers::{send_xcm_asset_hub, send_xcm_bridge_hub, utility_force_batch};
use sp_crypto_hashing::blake2_256;
use std::{io::Write, path::PathBuf};
use subxt::{OnlineClient, PolkadotConfig};

#[derive(Debug, Parser)]
#[command(name = "snowbridge-preimage", version, about, long_about = None)]
struct Cli {
    /// Output format of preimage
    #[arg(long, value_enum, default_value_t=Format::Hex)]
    format: Format,

    #[command(flatten)]
    api_endpoints: ApiEndpoints,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Register ERC20s on AH Kusama
    RegisterERC20s,
}

#[derive(Debug, Args)]
pub struct ApiEndpoints {
    #[arg(long, value_name = "URL")]
    bridge_hub_api: Option<String>,

    #[arg(long, value_name = "URL")]
    asset_hub_api: Option<String>,

    #[arg(long, value_name = "URL")]
    relay_api: Option<String>,
}

fn parse_eth_address(v: &str) -> Result<Address, String> {
    Address::parse_checksummed(v, None).map_err(|_| "invalid ethereum address".to_owned())
}

use std::str::FromStr;

fn parse_eth_address_without_validation(v: &str) -> Result<Address, String> {
    Address::from_str(v).map_err(|_| "invalid ethereum address".to_owned())
}

fn parse_hex_bytes32(v: &str) -> Result<FixedBytes<32>, String> {
    v.parse::<FixedBytes<32>>()
        .map_err(|_| "invalid 32-byte hex value".to_owned())
}

fn parse_hex_bytes(v: &str) -> Result<Bytes, String> {
    v.parse::<Bytes>()
        .map_err(|_| "invalid hex value".to_owned())
}

fn parse_units_polkadot(v: &str) -> Result<U128, String> {
    let amount = parse_units(v, POLKADOT_DECIMALS).map_err(|e| format!("{e}"))?;
    let amount: U256 = amount.into();
    let amount: U128 = amount.to::<U128>();
    Ok(amount)
}

fn parse_units_gwei(v: &str) -> Result<U256, String> {
    let amount = parse_units(v, "gwei").map_err(|e| format!("{e}"))?;
    Ok(amount.into())
}

fn parse_units_eth(v: &str) -> Result<U256, String> {
    let amount = parse_units(v, "ether").map_err(|e| format!("{e}"))?;
    Ok(amount.into())
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Format {
    Hex,
    Binary,
}

struct Context {
    bridge_hub_api: Box<OnlineClient<PolkadotConfig>>,
    asset_hub_api: Box<OnlineClient<PolkadotConfig>>,
    _relay_api: Box<OnlineClient<PolkadotConfig>>,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let bridge_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(
        cli.api_endpoints
            .bridge_hub_api
            .unwrap_or(BRIDGE_HUB_API.to_owned()),
    )
        .await?;

    let asset_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(
        cli.api_endpoints
            .asset_hub_api
            .unwrap_or(ASSET_HUB_API.to_owned()),
    )
        .await?;

    let relay_api: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(cli.api_endpoints.relay_api.unwrap_or(RELAY_API.to_owned())).await?;

    let context = Context {
        bridge_hub_api: Box::new(bridge_hub_api),
        asset_hub_api: Box::new(asset_hub_api),
        _relay_api: Box::new(relay_api),
    };

    let call = match &cli.command {
        Command::RegisterERC20s => {
            {
                let reg_call =
                    send_xcm_asset_hub(&context, commands::kusama_token_registrations()).await?;
                reg_call
            }
        }
    };

    let final_call = call;

    let preimage = final_call.encode();

    generate_chopsticks_script(&preimage, "chopsticks-execute-upgrade.js".into())?;

    eprintln!("Preimage Hash: 0x{}", hex::encode(blake2_256(&preimage)));
    eprintln!("Preimage Size: {}", preimage.len());

    match cli.format {
        Format::Hex => {
            println!("0x{}", hex::encode(preimage));
        }
        Format::Binary => {
            std::io::stdout().write_all(&preimage)?;
        }
    }

    Ok(())
}
