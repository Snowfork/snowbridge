mod asset_hub_runtime;
mod commands;
mod constants;
mod helpers;
mod relay_runtime;
mod xcm_helper;

use snowbridge_preimage_chopsticks::generate_chopsticks_script;
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::Encode;
use constants::{ASSET_HUB_API, RELAY_API};
use helpers::send_xcm_asset_hub;
use sp_crypto_hashing::blake2_256;
use std::io::Write;
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
    #[command(name = "register-erc20s-on-kusama")]
    RegisterERC20s,
}

#[derive(Debug, Args)]
pub struct ApiEndpoints {
    #[arg(long, value_name = "URL")]
    asset_hub_api: Option<String>,

    #[arg(long, value_name = "URL")]
    relay_api: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Format {
    Hex,
    Binary,
}

struct Context {
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

    let asset_hub_api: OnlineClient<PolkadotConfig> = OnlineClient::from_url(
        cli.api_endpoints
            .asset_hub_api
            .unwrap_or(ASSET_HUB_API.to_owned()),
    )
    .await?;

    let relay_api: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(cli.api_endpoints.relay_api.unwrap_or(RELAY_API.to_owned())).await?;

    let context = Context {
        asset_hub_api: Box::new(asset_hub_api),
        _relay_api: Box::new(relay_api),
    };

    let call = match &cli.command {
        Command::RegisterERC20s => {
            let reg_call = send_xcm_asset_hub(&context, commands::token_registrations()).await?;
            reg_call
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
