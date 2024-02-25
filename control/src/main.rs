mod runtime;
mod commands;
mod helpers;

use codec::Encode;
use clap::{Parser, Subcommand, ValueEnum};
use subxt::{OnlineClient, PolkadotConfig};
use std::io::Write;
use alloy_primitives::{Address, Bytes, FixedBytes};

#[derive(Debug, Parser)]
#[command(name = "snowbridge-control", version, about, long_about = None)]
struct Cli {
    /// Output format of preimage
    #[arg(value_enum, default_value_t=Format::Hex)]
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
        #[arg(long, value_name = "BYTES", value_parser=parse_hex_bytes)]
        initializer_params: Option<Bytes>,

        /// Maximum gas required by the initializer
        #[arg(long, value_name = "GAS")]
        initializer_gas: Option<u64>,
    }
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
    relay_api: &'a str,
    bridge_hub_api: &'a str,
}

struct Context {
    relay_api: Box<OnlineClient<PolkadotConfig>>,
    bridge_hub_api: Box<OnlineClient<PolkadotConfig>>,
}

#[cfg(feature = "rococo")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    relay_api: "wss://rococo-rpc.polkadot.io",
    bridge_hub_api: "wss://rococo-bridge-hub-rpc.polkadot.io",
};

#[cfg(feature = "kusama")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    relay_api: "wss://kusama-rpc.dwellir.com",
    bridge_hub_api: "wss://kusama-bridge-hub-rpc.polkadot.io",
};

#[cfg(feature = "polkadot")]
static CONFIG: StaticConfig<'static> = StaticConfig {
    relay_api: "wss://polkadot-rpc.dwellir.com",
    bridge_hub_api: "wss://polkadot-bridge-hub-rpc.polkadot.io",
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

	let bridge_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.bridge_hub_api)
		.await
		.expect("can not connect to bridgehub");

    let relay_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(CONFIG.relay_api)
        .await
        .expect("can not connect to relaychain");

    let context = Context {
        relay_api: Box::new(relay_api),
        bridge_hub_api: Box::new(bridge_hub_api)
    };

    let call = match &cli.command {
        Command::GatewayOperatingMode { mode } => {
            commands::gateway_operating_mode(&context, *mode).await?
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
            ).await?
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
