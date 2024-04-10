mod asset_hub_runtime;
mod bridge_hub_runtime;
mod commands;
mod constants;
mod helpers;
mod relay_runtime;

use alloy_primitives::{utils::parse_units, Address, Bytes, FixedBytes, U128, U256};
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::Encode;
use constants::{POLKADOT_DECIMALS, POLKADOT_SYMBOL};
use helpers::{force_xcm_version, send_xcm_asset_hub, send_xcm_bridge_hub, utility_batch};
use hex_literal::hex;
use std::{io::Write, path::PathBuf, process::exit};
use subxt::{OnlineClient, PolkadotConfig};

#[derive(Debug, Parser)]
#[command(name = "snowbridge-preimage", version, about, long_about = None)]
struct Cli {
    /// Output format of preimage
    #[arg(long, value_enum, default_value_t=Format::Hex)]
    format: Format,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize the bridge
    Initialize(InitializeArgs),
    /// Upgrade the Gateway contract
    Upgrade(UpgradeArgs),
    /// Change the gateway operating mode
    GatewayOperatingMode(GatewayOperatingModeArgs),
    /// Set pricing parameters
    PricingParameters(PricingParametersArgs),
    /// Set the checkpoint for the beacon light client
    ForceCheckpoint(ForceCheckpointArgs),
    Foo,
}

#[derive(Debug, Args)]
pub struct InitializeArgs {
    #[command(flatten)]
    gateway_operating_mode: GatewayOperatingModeArgs,
    #[command(flatten)]
    pricing_parameters: PricingParametersArgs,
    #[command(flatten)]
    force_checkpoint: ForceCheckpointArgs,
    #[command(flatten)]
    gateway_address: GatewayAddressArgs,
}

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Address of the logic contract
    #[arg(long, value_name = "ADDRESS", value_parser=parse_eth_address)]
    logic_address: Address,

    /// Hash of the code in the logic contract
    #[arg(long, value_name = "HASH", value_parser=parse_hex_bytes32)]
    logic_code_hash: FixedBytes<32>,

    /// Initialize the logic contract
    #[arg(long, requires_all=["initializer_params", "initializer_gas"])]
    initializer: bool,

    /// ABI-encoded params to pass to initializer
    #[arg(long, requires = "initializer", value_name = "BYTES", value_parser=parse_hex_bytes)]
    initializer_params: Option<Bytes>,

    /// Maximum gas required by the initializer
    #[arg(long, requires = "initializer", value_name = "GAS")]
    initializer_gas: Option<u64>,
}

#[derive(Debug, Args)]
pub struct GatewayOperatingModeArgs {
    /// Operating mode
    #[arg(long, value_enum)]
    gateway_operating_mode: GatewayOperatingModeEnum,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum GatewayOperatingModeEnum {
    Normal,
    RejectingOutboundMessages,
}

#[derive(Debug, Args)]
pub struct GatewayAddressArgs {
    /// Path to JSON file containing checkpoint
    #[arg(long, value_name = "ADDRESS")]
    pub gateway_address: Address,
}

#[derive(Debug, Args)]
pub struct ForceCheckpointArgs {
    /// Path to JSON file containing checkpoint
    #[arg(long, value_name = "FILE")]
    pub checkpoint: PathBuf,
}

#[derive(Debug, Args)]
pub struct PricingParametersArgs {
    /// Numerator for ETH/DOT Exchange rate
    ///
    /// For example, if the exchange rate is 1/400 (exchange 1 ETH for 400 DOT), then NUMERATOR should be 1.
    #[arg(long, value_name = "UINT")]
    pub exchange_rate_numerator: u64,
    /// Denominator for ETH/DOT Exchange rate
    ///
    /// For example, if the exchange rate is 1/400 (exchange 1 ETH for 400 DOT), then DENOMINATOR should be 400.
    #[arg(long, value_name = "UINT")]
    pub exchange_rate_denominator: u64,
    /// Numerator for Multiplier
    ///
    /// For example, if the multiplier is 4/3, then NUMERATOR should be 4.
    #[arg(long, value_name = "UINT")]
    pub multiplier_numerator: u64,
    /// Denominator for Multiplier
    ///
    /// For example, if the multiplier is 4/3, then DENOMINATOR should be 3.
    #[arg(long, value_name = "UINT")]
    pub multiplier_denominator: u64,
    /// Ether fee per unit of gas
    #[arg(long, value_name = "GWEI", value_parser = parse_units_gwei)]
    pub fee_per_gas: U256,
    /// Relayer reward for delivering messages to Polkadot
    #[arg(long, value_name = POLKADOT_SYMBOL, value_parser = parse_units_polkadot)]
    pub local_reward: U128,
    /// Relayer reward for delivering messages to Ethereum
    #[arg(long, value_name = "ETHER", value_parser = parse_units_eth)]
    pub remote_reward: U256,
}

fn parse_eth_address(v: &str) -> Result<Address, String> {
    Address::parse_checksummed(v, None).map_err(|_| "invalid ethereum address".to_owned())
}

fn parse_hex_bytes32(v: &str) -> Result<FixedBytes<32>, String> {
    v.parse::<FixedBytes<32>>()
        .map_err(|_| "invalid 32-byte hex value".to_owned())
}

fn parse_hex_bytes(v: &str) -> Result<Bytes, String> {
    v.parse::<Bytes>()
        .map_err(|_| "invalid hex value".to_owned())
}

fn parse_units_polkadot(v: &str) -> Result<u128, String> {
    let amount = parse_units(v, POLKADOT_DECIMALS).map_err(|e| format!("{e}"))?;
    let amount: U256 = amount.into();
    Ok(amount.to::<u128>())
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

struct StaticConfig<'a> {
    api: &'a str,
    asset_hub_api: &'a str,
}

struct Context {
    api: Box<OnlineClient<PolkadotConfig>>,
    asset_hub_api: Box<OnlineClient<PolkadotConfig>>,
}

#[cfg(feature = "rococo-local")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    api: "ws://127.0.0.1:11144",
    asset_hub_api: "ws://127.0.0.1:12144",
};

#[cfg(feature = "rococo")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    api: "wss://rococo-bridge-hub-rpc.polkadot.io",
    asset_hub_api: "wss://rococo-asset-hub-rpc.polkadot.io",
};

#[cfg(feature = "kusama")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    api: "wss://kusama-bridge-hub-rpc.polkadot.io",
    asset_hub_api: "wss://kusama-asset-hub-rpc.polkadot.io",
};

#[cfg(feature = "polkadot")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    api: "wss://polkadot-bridge-hub-rpc.polkadot.io",
    asset_hub_api: "wss://polkadot-asset-hub-rpc.polkadot.io",
};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
    }
}

use std::{fs::File, io::Read};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.api)
        .await
        .expect("can not connect to bridgehub");

    let asset_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.asset_hub_api)
        .await
        .expect("can not connect to assethub");

    let context = Context {
        api: Box::new(api),
        asset_hub_api: Box::new(asset_hub_api),
    };

    let call = match &cli.command {
        Command::ForceCheckpoint(params) => {
            let call = commands::force_checkpoint(params);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::Initialize(params) => {
            let (set_pricing_parameters, set_ethereum_fee) =
                commands::pricing_parameters(&context, &params.pricing_parameters).await?;
            let call1 = send_xcm_bridge_hub(
                &context,
                vec![
                    commands::force_checkpoint(&params.force_checkpoint),
                    commands::gateway_operating_mode(&params.gateway_operating_mode),
                    commands::set_gateway_address(&params.gateway_address),
                    set_pricing_parameters,
                ],
            )
            .await?;
            let call2 =
                send_xcm_asset_hub(&context, vec![force_xcm_version(), set_ethereum_fee]).await?;
            utility_batch(vec![call1, call2])
        }
        Command::GatewayOperatingMode(params) => {
            let call = commands::gateway_operating_mode(params);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::Upgrade(params) => {
            let call = commands::upgrade(params);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::PricingParameters(params) => {
            let (set_pricing_parameters, set_ethereum_fee) =
                commands::pricing_parameters(&context, params).await?;
            let call1 = send_xcm_bridge_hub(&context, vec![set_pricing_parameters]).await?;
            let call2 = send_xcm_asset_hub(&context, vec![set_ethereum_fee]).await?;
            utility_batch(vec![call1, call2])
        }
        Command::Foo => {
            let mut file = File::open("preimage-upgrade-1.2.0.bin").expect("File not found");
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf).expect("Failed to read the file");

            let f = buf.encode();
            println!("{}", hex::encode(f));
            exit(0);
        }
    };

    let preimage = call.encode();

    match cli.format {
        Format::Hex => {
            println!("0x{}", hex::encode(preimage));
        }
        Format::Binary => {
            std::io::stdout()
                .write_all(&preimage)
                .expect("write stdout");
        }
    }

    Ok(())
}
