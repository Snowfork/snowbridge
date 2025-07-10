mod estimator;

use clap::{Parser, Subcommand, ValueEnum};
use serde_json;
use std::process;
use crate::estimator::EstimatorArgs;
use crate::estimator::EstimatorError;

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

    // Parse and validate arguments
    let args = match parse_args(cli) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
//
    //// Run the estimation
    //match run_estimation(args).await {
    //    Ok(output) => {
    //        println!("{}", output);
    //    }
    //    Err(e) => {
    //        eprintln!("Error: {}", e);
    //        process::exit(1);
    //    }
    //}
}

fn parse_args(cli: Cli) -> Result<EstimatorArgs, EstimatorError> {
    Ok(EstimatorArgs::new(cli.env, cli.command)?)
}
//
//async fn run_estimation(
//    (args, output_format): (EstimatorArgs, OutputFormat),
//) -> Result<String, EstimatorError> {
//
//    //format_output(&estimation, output_format)
//}

//fn format_output(
//    estimation: &crate::types::GasEstimation,
//    format: OutputFormat,
//) -> Result<String, EstimatorError> {
//    match format {
//        OutputFormat::Json => {
//            serde_json::to_string_pretty(estimation)
//                .map_err(|e| EstimatorError::ConfigError(format!("JSON serialization failed: {}", e)))
//        }
//    }
//}
