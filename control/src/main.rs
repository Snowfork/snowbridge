mod governance;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::fs;

use governance::GovernanceCommand;


use subxt::{OnlineClient, PolkadotConfig};
use toml;

#[derive(Debug, Parser)]
#[command(name = "snowbridge-control", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// does testing things
    Governance {
        #[command(subcommand)]
        command: GovernanceCommand,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Network {
    Rococo,
    Kusama,
    Polkadot,
}



    pub static STATIC_CONFIG: &str = r#"
    [networks]
    [networks.rococo]
    relay_api = "wss://rococo-rpc.polkadot.io"
    bridge_hub_api = "wss://rococo-bridge-hub-rpc.polkadot.io"

    [networks.kusama]
    relay_api = "wss://rpc.dotters.network/kusama"
    bridge_hub_api = "wss://kusama-bridge-hub-rpc.polkadot.io"

    [networks.polkadot]
    relay_api = "wss://polkadot-rpc.dwellir.com"
    bridge_hub_api = "wss://polkadot-bridge-hub-rpc.polkadot.io"
    "#;


struct Config<'a> {
    relay_api: &'a str,
    bridge_hub_api: &'a str,
}

struct Context {
    relay_api: Box<OnlineClient<PolkadotConfig>>,
    bridge_hub_api: Box<OnlineClient<PolkadotConfig>>,
}

static ROCOCO_CONFIG: Config<'static> = Config {
    relay_api: "wss://rococo-rpc.polkadot.io",
    bridge_hub_api: "wss://rococo-bidge-hub-rpc.polkadot.io",
};

static KUSAMA_CONFIG: Config<'static> = Config {
    relay_api: "wss://kusama-rpc.dwellir.com",
    bridge_hub_api: "wss://kusama-bridge-hub-rpc.polkadot.io",
};

static POLKADOT_CONFIG: Config<'static> = Config {
    relay_api: "wss://polkadot-rpc.dwellir.com",
    bridge_hub_api: "wss://polkadot-bridge-hub-rpc.polkadot.io",
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    #[cfg(feature = "rococo")]
    let config = &ROCOCO_CONFIG;

    #[cfg(feature = "kusama")]
    let config = &KUSAMA_CONFIG;

    #[cfg(feature = "polkadot")]
    let config = &POLKADOT_CONFIG;

	let bridge_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(config.bridge_hub_api)
		.await
		.expect("can not connect to bridgehub");

    let relay_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(config.relay_api)
        .await
        .expect("can not connect to relaychain");

    let context = Context {
        relay_api: Box::new(relay_api),
        bridge_hub_api: Box::new(bridge_hub_api)
    };

    match &cli.command {
        Command::Governance { command } => {
            governance::run(&context, command).await.expect("Foo");
        }
    }

    Ok(())
}
