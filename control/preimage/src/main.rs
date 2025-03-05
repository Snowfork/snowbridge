mod asset_hub_runtime;
mod bridge_hub_runtime;
mod chopsticks;
mod commands;
mod constants;
mod helpers;
mod relay_runtime;
mod treasury_commands;

use alloy_primitives::{utils::parse_units, Address, Bytes, FixedBytes, U128, U256};
use chopsticks::generate_chopsticks_script;
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::Encode;
use constants::{ASSET_HUB_API, BRIDGE_HUB_API, POLKADOT_DECIMALS, POLKADOT_SYMBOL, RELAY_API};
use helpers::{force_xcm_version, send_xcm_asset_hub, send_xcm_bridge_hub, utility_force_batch};
use sp_crypto_hashing::blake2_256;
use std::{io::Write, path::PathBuf};
use subxt::{OnlineClient, PolkadotConfig};

#[cfg(any(feature = "westend", feature = "paseo"))]
use crate::helpers::sudo;

#[derive(Debug, Parser)]
#[command(name = "snowbridge-preimage", version, about, long_about = None)]
struct Cli {
    /// Output format of preimage
    #[arg(long, value_enum, default_value_t=Format::Hex)]
    format: Format,

    /// Wrap preimage in a sudo call
    #[cfg(any(feature = "westend", feature = "paseo"))]
    #[arg(long, default_value_t = false)]
    sudo: bool,

    #[command(flatten)]
    api_endpoints: ApiEndpoints,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize the bridge
    Initialize(InitializeArgs),
    /// Update the asset on AssetHub
    UpdateAsset(UpdateAssetArgs),
    /// Upgrade the Gateway contract
    Upgrade(UpgradeArgs),
    /// Change the gateway operating mode
    GatewayOperatingMode(GatewayOperatingModeArgs),
    /// Set pricing parameters
    PricingParameters(PricingParametersArgs),
    /// Set the checkpoint for the beacon light client
    ForceCheckpoint(ForceCheckpointArgs),
    /// Set the checkpoint for the beacon light client
    HaltBridge(HaltBridgeArgs),
    /// Register Ether
    RegisterEther(RegisterEtherArgs),
    /// Treasury proposal
    TreasuryProposal2024(TreasuryProposal2024Args),
    /// Governance update 202501
    GovUpdate202501(GovUpdate202501Args),
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
    #[command(flatten)]
    register_ether: RegisterEtherArgs,
}

#[derive(Debug, Args)]
pub struct UpdateAssetArgs {
    /// Chain ID of the Ethereum chain bridge from.
    #[arg(long, value_name = "ADDRESS", value_parser=parse_eth_address_without_validation)]
    contract_id: Address,
    /// The asset display name, e.g. Wrapped Ether
    #[arg(long, value_name = "ASSET_DISPLAY_NAME")]
    name: String,
    /// The asset symbol, e.g. WETH
    #[arg(long, value_name = "ASSET_SYMBOL")]
    symbol: String,
    /// The asset's number of decimal places.
    #[arg(long, value_name = "DECIMALS")]
    decimals: u8,
    /// The minimum balance of the asset.
    #[arg(long, value_name = "MIN_BALANCE")]
    min_balance: u128,
    /// Should the asset be sufficient.
    #[arg(long, value_name = "IS_SUFFICIENT")]
    is_sufficient: bool,
    /// Should the asset be frozen.
    #[arg(long, value_name = "IS_FROZEN")]
    is_frozen: bool,
}

#[derive(Debug, Args)]
pub struct UpgradeArgs {
    /// Address of the logic contract
    #[arg(long, value_name = "ADDRESS", value_parser=parse_eth_address)]
    logic_address: Address,

    /// Hash of the code in the logic contract
    #[arg(long, value_name = "HASH", value_parser=parse_hex_bytes32)]
    logic_code_hash: FixedBytes<32>,

    /// ABI-encoded params to pass to initializer
    #[arg(long, value_name = "BYTES", value_parser=parse_hex_bytes)]
    initializer_params: Bytes,

    /// Maximum gas required by the initializer
    #[arg(long, value_name = "GAS")]
    initializer_gas: u64,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum OperatingModeEnum {
    Normal,
    Halted,
}

#[derive(Debug, Args)]
pub struct GatewayAddressArgs {
    /// Address of the contract on Ethereum
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

#[derive(Debug, Args)]
pub struct HaltBridgeArgs {
    /// Halt the Ethereum gateway, blocking message from Ethereum to Polkadot in the Ethereum
    /// contract.
    #[arg(long, value_name = "HALT_GATEWAY")]
    gateway: bool,
    /// Halt the Ethereum Inbound Queue, blocking messages from BH to AH.
    #[arg(long, value_name = "HALT_INBOUND_QUEUE")]
    inbound_queue: bool,
    /// Halt the Ethereum Outbound Queue, blocking message from AH to BH.
    #[arg(long, value_name = "HALT_OUTBOUND_QUEUE")]
    outbound_queue: bool,
    /// Halt the Ethereum client, blocking consensus updates to the ligth client.
    #[arg(long, value_name = "HALT_ETHEREUM_CLIENT")]
    ethereum_client: bool,
    /// Set the AH to Ethereum fee to a high amount, effectively blocking messages from AH ->
    /// Ethereum.
    #[arg(long, value_name = "ASSETHUB_MAX_FEE")]
    assethub_max_fee: bool,
    /// Halt all parts of the bridge
    #[arg(long, value_name = "HALT_SNOWBRIDGE")]
    all: bool,
}

#[derive(Debug, Args)]
pub struct TreasuryProposal2024Args {
    /// Beneficiary address
    #[arg(long, value_name = "ADDRESS", value_parser=parse_hex_bytes32)]
    beneficiary: FixedBytes<32>,
}

#[derive(Debug, Args)]
pub struct GovUpdate202501Args {
    #[command(flatten)]
    pricing_parameters: PricingParametersArgs,
    #[command(flatten)]
    register_ether: RegisterEtherArgs,
}

#[derive(Debug, Args)]
pub struct RegisterEtherArgs {
    /// The minimum balance of the Ether asset that users are allowed to hold
    #[arg(long, value_name = "WEI", default_value_t = 1u128)]
    ether_min_balance: u128,
    /// The Ether asset display name
    #[arg(long, value_name = "ASSET_DISPLAY_NAME", default_value_t = String::from("Ether"))]
    ether_name: String,
    /// The Ether asset symbol
    #[arg(long, value_name = "ASSET_SYMBOL", default_value_t = String::from("ETH"))]
    ether_symbol: String,
    /// The Ether asset's number of decimal places
    #[arg(long, value_name = "DECIMALS", default_value_t = 18u8)]
    ether_decimals: u8,
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
        Command::ForceCheckpoint(params) => {
            let call = commands::force_checkpoint(params);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::Initialize(params) => {
            let (set_pricing_parameters, set_ethereum_fee) =
                commands::pricing_parameters(&context, &params.pricing_parameters).await?;
            let bridge_hub_call = send_xcm_bridge_hub(
                &context,
                vec![
                    commands::set_gateway_address(&params.gateway_address),
                    set_pricing_parameters,
                    commands::gateway_operating_mode(
                        &params.gateway_operating_mode.gateway_operating_mode,
                    ),
                    commands::force_checkpoint(&params.force_checkpoint),
                ],
            )
            .await?;
            let (register_ether_call, set_ether_metadata_call) =
                commands::register_ether(&params.register_ether);
            let asset_hub_call = send_xcm_asset_hub(
                &context,
                vec![
                    register_ether_call,
                    set_ether_metadata_call,
                    force_xcm_version(),
                    set_ethereum_fee,
                ],
            )
            .await?;
            utility_force_batch(vec![bridge_hub_call, asset_hub_call])
        }
        Command::UpdateAsset(params) => {
            send_xcm_asset_hub(
                &context,
                vec![
                    commands::make_asset_sufficient(params),
                    commands::force_set_metadata(params),
                ],
            )
            .await?
        }
        Command::GatewayOperatingMode(params) => {
            let call = commands::gateway_operating_mode(&params.gateway_operating_mode);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::Upgrade(params) => {
            let call = commands::upgrade(params);
            send_xcm_bridge_hub(&context, vec![call]).await?
        }
        Command::PricingParameters(params) => {
            let (set_pricing_parameters, set_ethereum_fee) =
                commands::pricing_parameters(&context, params).await?;
            let bridge_hub_call =
                send_xcm_bridge_hub(&context, vec![set_pricing_parameters]).await?;
            let asset_hub_call = send_xcm_asset_hub(&context, vec![set_ethereum_fee]).await?;
            utility_force_batch(vec![bridge_hub_call, asset_hub_call])
        }
        Command::HaltBridge(params) => {
            let mut bh_calls = vec![];
            let mut ah_calls = vec![];
            let mut halt_all = params.all;
            // if no individual option specified, assume halt the whole bridge.
            if !params.gateway
                && !params.inbound_queue
                && !params.outbound_queue
                && !params.ethereum_client
                && !params.assethub_max_fee
            {
                halt_all = true;
            }
            if params.gateway || halt_all {
                bh_calls.push(commands::gateway_operating_mode(
                    &GatewayOperatingModeEnum::RejectingOutboundMessages,
                ));
            }
            if params.inbound_queue || halt_all {
                bh_calls.push(commands::inbound_queue_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.outbound_queue || halt_all {
                bh_calls.push(commands::outbound_queue_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.ethereum_client || halt_all {
                bh_calls.push(commands::ethereum_client_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.assethub_max_fee || halt_all {
                ah_calls.push(commands::set_assethub_fee(u128::MAX));
            }
            if bh_calls.len() > 0 && ah_calls.len() == 0 {
                send_xcm_bridge_hub(&context, bh_calls).await?
            } else if ah_calls.len() > 0 && bh_calls.len() == 0 {
                send_xcm_asset_hub(&context, ah_calls).await?
            } else {
                let call1 = send_xcm_bridge_hub(&context, bh_calls).await?;
                let call2 = send_xcm_asset_hub(&context, ah_calls).await?;
                utility_force_batch(vec![call1, call2])
            }
        }
        Command::RegisterEther(params) => {
            let (register_ether_call, set_ether_metadata_call) = commands::register_ether(&params);
            send_xcm_asset_hub(&context, vec![register_ether_call, set_ether_metadata_call]).await?
        }
        Command::TreasuryProposal2024(params) => treasury_commands::treasury_proposal(&params),
        Command::GovUpdate202501(GovUpdate202501Args {
            pricing_parameters,
            register_ether,
        }) => {
            let (set_pricing_parameters, set_ethereum_fee) =
                commands::pricing_parameters(&context, pricing_parameters).await?;

            let bh_set_pricing_call =
                send_xcm_bridge_hub(&context, vec![set_pricing_parameters]).await?;

            let ah_set_pricing_call = send_xcm_asset_hub(&context, vec![set_ethereum_fee]).await?;

            let (register_ether_call, set_ether_metadata_call) =
                commands::register_ether(&register_ether);
            let ah_register_ether_call =
                send_xcm_asset_hub(&context, vec![register_ether_call, set_ether_metadata_call])
                    .await?;

            utility_force_batch(vec![
                bh_set_pricing_call,
                ah_set_pricing_call,
                ah_register_ether_call,
            ])
        }
    };

    #[cfg(any(feature = "westend", feature = "paseo"))]
    let final_call = if cli.sudo { sudo(Box::new(call)) } else { call };
    #[cfg(not(any(feature = "westend", feature = "paseo")))]
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
