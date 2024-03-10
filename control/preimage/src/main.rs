mod bridge_hub_runtime;
mod asset_hub_runtime;
mod relay_runtime;
mod commands;
mod helpers;
mod constants;
mod fees;

use codec::Encode;
use clap::{Parser, Subcommand, ValueEnum, Args};
use helpers::{wrap_calls_asset_hub, utility_batch};
use subxt::{OnlineClient, PolkadotConfig};
use std::io::Write;
use alloy_primitives::{Address, Bytes, FixedBytes};

#[derive(Debug, Parser)]
#[command(name = "snowbridge-control", version, about, long_about = None)]
struct Cli {
    /// Output format of preimage
    #[arg(long, value_enum, default_value_t=Format::Hex)]
    format: Format,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Change the gateway operating mode
    GatewayOperatingMode {
        /// Operating mode
        #[arg(value_enum)]
        mode: GatewayOperatingModeArg,
    },
    /// Upgrade the Gateway contract
    Upgrade {

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
    },
    /// Set pricing parameters
    PricingParameters(PricingParametersArgs),
    ForceCheckpoint(ForceCheckpointArgs),
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
    /// Ether fee per unit of gas
    #[arg(long, value_name = "GWEI")]
    pub fee_per_gas: u64,
    /// Relayer reward for delivering messages to Polkadot
    #[arg(long, value_name = "PLANCK")]
    pub local_reward: U128,
    /// Relayer reward for delivering messages to Ethereum
    #[arg(long, value_name = "WEI")]
    pub remote_reward: U256,
}

fn parse_eth_address(v: &str) -> Result<Address, String> {
    Address::parse_checksummed(v, None).map_err(|_| {
        "invalid ethereum address".to_owned()
    })
}

fn parse_hex_bytes32(v: &str) -> Result<FixedBytes<32>, String> {
    v.parse::<FixedBytes<32>>().map_err(|_| {
        "invalid 32-byte hex value".to_owned()
    })
}

fn parse_hex_bytes(v: &str) -> Result<Bytes, String> {
    v.parse::<Bytes>().map_err(|_| {
        "invalid hex value".to_owned()
    })
}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Format {
    Hex,
    Binary,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum GatewayOperatingModeArg {
    Normal,
    RejectingOutboundMessages,
}

struct StaticConfig<'a> {
    api: &'a str,
    asset_hub_api: &'a str,
}

struct Context {
    api: Box<OnlineClient<PolkadotConfig>>,
    asset_hub_api: Box<OnlineClient<PolkadotConfig>>,
}

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

	let api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.api)
		.await
		.expect("can not connect to bridgehub");

    let asset_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.asset_hub_api)
    .await
    .expect("can not connect to assethub");


    let context = Context {
        api: Box::new(api),
        asset_hub_api: Box::new(asset_hub_api)
    };

    let call = match &cli.command {
        Command::ForceCheckpoint(ForceCheckpointArgs { checkpoint }) => {
            let mut file = File::open(checkpoint).expect("File not found");
            let mut data = String::new();
            file.read_to_string(&mut data).expect("Failed to read the file");
            let checkpoint: snowbridge_beacon_primitives::CheckpointUpdate<512> = serde_json::from_str(&data).unwrap();
            let call = commands::force_checkpoint(checkpoint);
            wrap_calls(&context, vec![call]).await?
        },
        Command::Initialize {
            gateway_operating_mode: GatewayOperatingModeArgs { gateway_operating_mode },
            pricing_parameters: PricingParametersArgs { exchange_rate_numerator, exchange_rate_denominator, fee_per_gas, local_reward, remote_reward },
        } => {
            let call1 = commands::gateway_operating_mode(*gateway_operating_mode);
            let calls2 = commands::pricing_parameters(&context, *exchange_rate_numerator, *exchange_rate_denominator, *fee_per_gas, *local_reward, *remote_reward).await?;
            wrap_calls(&context, vec![call1, calls2.0]).await?
        },
        Command::GatewayOperatingMode(GatewayOperatingModeArgs { gateway_operating_mode }) => {
            let call = commands::gateway_operating_mode(*gateway_operating_mode);
            wrap_calls(&context, vec![call]).await?
        },
        Command::Upgrade { logic_address, logic_code_hash, initializer, initializer_params, initializer_gas} => {
            let initializer = if *initializer {
                Some((initializer_params.as_ref().unwrap().clone(), initializer_gas.unwrap()))
            } else {
                None
            };
            commands::upgrade(
                &context,
                *logic_address,
                *logic_code_hash,
                initializer
            );
            wrap_calls(&context, vec![call]).await?
        },
        Command::PricingParameters(PricingParametersArgs { exchange_rate_numerator, exchange_rate_denominator, fee_per_gas, local_reward, remote_reward }) => {
            let calls = commands::pricing_parameters(&context, *exchange_rate_numerator, *exchange_rate_denominator, *fee_per_gas, *local_reward, *remote_reward).await?;
            let call1 = wrap_calls(&context, vec![calls.0]).await?;
            let call2 = wrap_calls_asset_hub(&context, vec![calls.1]).await?;
            let call = utility_batch(vec![call1, call2]);
            call
        }
    };

    let preimage = call.encode();

    match cli.format {
        Format::Hex => {
            println!("0x{}", hex::encode(preimage));
        },
        Format::Binary => {
            std::io::stdout().write_all(&preimage).expect("write stdout");
        }
    }

    Ok(())
}
