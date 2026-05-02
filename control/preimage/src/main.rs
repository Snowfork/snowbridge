mod asset_hub_runtime;
mod bridge_hub_runtime;
mod commands;
mod constants;
mod helpers;
#[allow(unused)]
mod relay_runtime;
mod treasury_commands;

use alloy_primitives::{address, utils::parse_units, Address, Bytes, FixedBytes, U128, U256};
use clap::{Args, Parser, Subcommand, ValueEnum};
use codec::Encode;
use constants::{ASSET_HUB_API, BRIDGE_HUB_API, POLKADOT_DECIMALS, POLKADOT_SYMBOL, RELAY_API};
use helpers::{force_xcm_version, send_xcm_asset_hub, send_xcm_bridge_hub, utility_force_batch};
use snowbridge_preimage_chopsticks::generate_chopsticks_script;
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
    /// Register PNA
    RegisterPnaBatch202503,
    /// Register all ERC20 tokens metadata
    RegisterErc20TokenMetadata,
    /// Upgrade to V2
    UpgradeV2,
    /// Replay failed XCM messages from September 2025
    ReplaySep2025,
    /// Mint refund for failed Hydration→Ethereum transfer (Feb 2026)
    MintFeb2026,
    /// Set BridgeHubEthereumBaseFeeV2 on Paseo
    SetPaseoFeeV2,
    /// Upgrade to FiatShamir on Polkadot
    #[command(alias = "upgrade-202603")]
    Upgrade202603,
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
    /// Halt the Ethereum Gateway contract (both V1 and V2 paths). Sends
    /// `Command::SetOperatingMode(Halted)` via both V1 and V2 system pallets so the halt
    /// is delivered via whichever outbound queue is live. Once processed on Ethereum,
    /// this blocks `v2_sendMessage`, `v2_registerToken`, and V1 `sendToken`/`sendMessage`
    /// on the Gateway. Delivery is relayer-dependent.
    #[arg(long, value_name = "HALT_GATEWAY")]
    gateway: bool,
    /// Halt both V1 and V2 inbound-queue pallets on BridgeHub, blocking processing of
    /// Ethereum -> Polkadot messages. For surgical halts of a single version, use
    /// `--inbound-queue-v1` or `--inbound-queue-v2`.
    #[arg(long, value_name = "HALT_INBOUND_QUEUE")]
    inbound_queue: bool,
    /// Halt only the V1 inbound-queue pallet on BridgeHub.
    #[arg(long, value_name = "HALT_INBOUND_QUEUE_V1")]
    inbound_queue_v1: bool,
    /// Halt only the V2 inbound-queue pallet on BridgeHub.
    #[arg(long, value_name = "HALT_INBOUND_QUEUE_V2")]
    inbound_queue_v2: bool,
    /// Halt AssetHub -> Ethereum outbound traffic. Halts the V1 outbound-queue pallet
    /// on BridgeHub AND the system-frontend pallet on AssetHub; the latter short-circuits
    /// the AssetHub->Ethereum `PausableExporter` for both V1 and V2 at the XcmRouter
    /// layer (V2's `outbound-queue-v2` has no local halt, so the frontend halt is the
    /// primary V2 outbound lever).
    #[arg(long, value_name = "HALT_OUTBOUND_QUEUE")]
    outbound_queue: bool,
    /// Halt the Ethereum beacon light client, blocking new beacon-header ingestion.
    /// Note: this does NOT propagate into the `Verifier::verify` trait impl that
    /// downstream consumers (`inbound-queue(-v2)::submit`,
    /// `outbound-queue-v2::submit_delivery_receipt`) call — those verify against
    /// already-stored finalised state. Halt those consumers individually to block
    /// proof-consuming flows during a suspected beacon compromise.
    #[arg(long, value_name = "HALT_ETHEREUM_CLIENT")]
    ethereum_client: bool,
    /// Set the AssetHub -> Ethereum outbound fee to `u128::MAX` for both V1
    /// (`BridgeHubEthereumBaseFee`) and V2 (`BridgeHubEthereumBaseFeeV2`) storage
    /// items, effectively deterring user sends via fee pricing. Complementary to the
    /// system-frontend halt; does not block at the router layer.
    #[arg(long, value_name = "ASSETHUB_MAX_FEE")]
    assethub_max_fee: bool,
    /// Halt all parts of the bridge (equivalent to passing every other flag).
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

use hex_literal::hex;
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
    _asset_hub_api: Box<OnlineClient<PolkadotConfig>>,
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
        _asset_hub_api: Box::new(asset_hub_api),
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
                && !params.inbound_queue_v1
                && !params.inbound_queue_v2
                && !params.outbound_queue
                && !params.ethereum_client
                && !params.assethub_max_fee
            {
                halt_all = true;
            }
            // Gateway halt commands must be enqueued BEFORE any local outbound-queue
            // halt takes effect, otherwise the SetOperatingMode command cannot be
            // committed for delivery to Ethereum. Push both V1 and V2 variants so the
            // halt is delivered via whichever outbound queue is operational.
            if params.gateway || halt_all {
                bh_calls.push(commands::gateway_operating_mode(
                    &GatewayOperatingModeEnum::RejectingOutboundMessages,
                ));
                bh_calls.push(commands::gateway_operating_mode_v2(
                    &GatewayOperatingModeEnum::RejectingOutboundMessages,
                ));
            }
            if params.inbound_queue || params.inbound_queue_v1 || halt_all {
                bh_calls.push(commands::inbound_queue_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.inbound_queue || params.inbound_queue_v2 || halt_all {
                bh_calls.push(commands::inbound_queue_v2_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.outbound_queue || halt_all {
                // V1 local halt on BridgeHub. V2's outbound-queue-v2 has no local halt;
                // the system-frontend halt below is the effective V2 outbound lever.
                bh_calls.push(commands::outbound_queue_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
                // system-frontend halt on AssetHub: short-circuits the PausableExporter
                // wrapping the AH->Ethereum router, blocking both V1 and V2 exports at
                // the source regardless of user or parachain origin.
                ah_calls.push(commands::system_frontend_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.ethereum_client || halt_all {
                bh_calls.push(commands::ethereum_client_operating_mode(
                    &OperatingModeEnum::Halted,
                ));
            }
            if params.assethub_max_fee || halt_all {
                // Set both V1 and V2 AssetHub outbound fee storage items to u128::MAX.
                ah_calls.push(commands::set_assethub_fee(u128::MAX));
                ah_calls.push(commands::set_assethub_fee_v2(u128::MAX));
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
        Command::RegisterPnaBatch202503 => {
            #[cfg(not(feature = "polkadot"))]
            panic!("RegisterPnaBatch202503 only for polkadot runtime.");

            #[cfg(feature = "polkadot")]
            {
                let reg_call =
                    send_xcm_bridge_hub(&context, commands::token_registrations()).await?;
                reg_call
            }
        }
        Command::RegisterErc20TokenMetadata => {
            #[cfg(not(feature = "polkadot"))]
            panic!("RegisterErc20TokenMetadata only for polkadot runtime.");

            #[cfg(feature = "polkadot")]
            {
                let metadata_calls = commands::register_erc20_token_metadata();
                let reg_call = commands::frequency_token_registrations();
                utility_force_batch(vec![
                    send_xcm_asset_hub(&context, metadata_calls).await?,
                    send_xcm_bridge_hub(&context, reg_call).await?,
                ])
            }
        }
        Command::UpgradeV2 => {
            #[cfg(not(feature = "polkadot"))]
            panic!("UpgradeV2 only for polkadot runtime.");

            #[cfg(feature = "polkadot")]
            {
                // Upgrade logic gateway on BH
                let upgrade_call = commands::upgrade(&UpgradeArgs {
                    logic_address: address!("8a887783E945233d51881e06835Ec78A8b575eCe"),
                    logic_code_hash: FixedBytes::from_slice(&hex!(
                        "cbabd7683b33e7d8f4b143def2d712999961b306e1f98782016439293d673849"
                    )),
                    initializer_params: Default::default(),
                    initializer_gas: 100000,
                });
                let bh_xcm_call = send_xcm_bridge_hub(&context, vec![upgrade_call]).await?;

                // Set bound fee to 0.1 DOT on AH
                let outbound_fee_call = commands::set_assethub_fee_v2(1_000_000_000);
                let ah_xcm_call = send_xcm_asset_hub(&context, vec![outbound_fee_call]).await?;

                utility_force_batch(vec![bh_xcm_call, ah_xcm_call])
            }
        }
        Command::ReplaySep2025 => {
            let asset_hub_call = commands::replay_sep_2025_xcm();
            send_xcm_asset_hub(&context, vec![asset_hub_call]).await?
        }
        Command::MintFeb2026 => {
            let bridge_hub_call = commands::mint_feb_2026_xcm();
            send_xcm_bridge_hub(&context, vec![bridge_hub_call]).await?
        }
        Command::SetPaseoFeeV2 => {
            #[cfg(not(feature = "paseo"))]
            panic!("SetPaseoFeeV2 only for paseo runtime.");

            #[cfg(feature = "paseo")]
            {
                // Set bound fee to 0.1 DOT (same as Polkadot V2) on AH
                let outbound_fee_call = commands::set_assethub_fee_v2(1_000_000_000);
                send_xcm_asset_hub(&context, vec![outbound_fee_call]).await?
            }
        }
        Command::Upgrade202603 => {
            #[cfg(not(feature = "polkadot"))]
            panic!("Upgrade202603 only for polkadot runtime.");

            #[cfg(feature = "polkadot")]
            {
                // Upgrade logic gateway
                let upgrade_call = commands::upgrade(&UpgradeArgs {
                    logic_address: address!("36e74FCAAcb07773b144Ca19Ef2e32Fc972aC50b"),
                    logic_code_hash: FixedBytes::from_slice(&hex!(
                        "e3cfcc0042ad4c819c627fb2a84ba0822d67747a8618a4e1c4eb0c5112b17903"
                    )),
                    initializer_params: Default::default(),
                    initializer_gas: 100000,
                });
                send_xcm_bridge_hub(&context, vec![upgrade_call]).await?
            }
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
