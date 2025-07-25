#[cfg(feature = "local")]
use asset_hub_westend_local_runtime::runtime_types::{
    bounded_collections::bounded_vec::BoundedVec,
    sp_weights::weight_v2::Weight,
    staging_xcm::v5::{
        asset::{
            Asset,
            AssetFilter::{Definite, Wild},
            AssetId, Assets,
            Fungibility::Fungible,
            WildAsset::AllCounted,
        },
        junction::{
            Junction::{AccountId32, AccountKey20, GlobalConsensus, PalletInstance},
            NetworkId::{self, Ethereum},
        },
        junctions::Junctions::{Here, X1, X2},
        location::Location,
        traits::Outcome,
        Hint::AssetClaimer,
        Instruction::{
            DepositAsset, DescendOrigin, ExchangeAsset, PayFees, RefundSurplus,
            ReserveAssetDeposited, SetHints, Transact, UniversalOrigin, WithdrawAsset,
        },
        Xcm,
    },
    xcm::v3::OriginKind,
    xcm::VersionedAssets,
    xcm::{VersionedAssetId, VersionedLocation, VersionedXcm},
};
use codec::DecodeLimit;
use hex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sp_core::parameter_types;
use sp_core::H256;
use sp_runtime::AccountId32 as RuntimeAccountId32;
use std::env;
use subxt::{config::DefaultExtrinsicParams, Config, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

use crate::penpal;

lazy_static! { // TODO extract to config file or something
    pub static ref ASSET_HUB_WS_URL: String = {
        if let Ok(val) = env::var("ASSET_HUB_WS_URL") {
            val
        } else {
            "ws://127.0.0.1:12144".to_string()
        }
    };

    pub static ref BRIDGE_HUB_WS_URL: String = {
        if let Ok(val) = env::var("BRIDGE_HUB_WS_URL") {
            val
        } else {
            "ws://127.0.0.1:11144".to_string()
        }
    };

    pub static ref PENPAL_WS_URL: String = {
        if let Ok(val) = env::var("PENPAL_WS_URL") {
            val
        } else {
            "ws://127.0.0.1:13144".to_string()
        }
    };
}

const CHAIN_ID: u64 = 11155111; // TODO switch on env
const INBOUND_PALLET_V2: u8 = 91; // TODO switch on env
const ASSET_HUB_PARA_ID: u32 = 1000;
const BRIDGE_HUB_PARA_ID: u32 = 1002;
const PENPAL_PARA_ID: u32 = 2000;

// Register token constants
const CREATE_ASSET_DEPOSIT: u128 = 100_000_000_000;
const CREATE_ASSET_CALL: [u8; 2] = [53, 0]; // ForeignAssets::create call index
const MINIMUM_DEPOSIT: u128 = 1;

/// Custom config that works with Statemint
pub enum AssetHubConfig {}

impl Config for AssetHubConfig {
    type AccountId = <PolkadotConfig as Config>::AccountId;
    type Address = <PolkadotConfig as Config>::Address;
    type Signature = <PolkadotConfig as Config>::Signature;
    type Hasher = <PolkadotConfig as Config>::Hasher;
    type Header = <PolkadotConfig as Config>::Header;
    type ExtrinsicParams = DefaultExtrinsicParams<AssetHubConfig>;
    type AssetId = <PolkadotConfig as Config>::AssetId;
}

/// Custom config for Penpal
pub enum PenpalConfig {}

impl Config for PenpalConfig {
    type AccountId = <PolkadotConfig as Config>::AccountId;
    type Address = <PolkadotConfig as Config>::Address;
    type Signature = <PolkadotConfig as Config>::Signature;
    type Hasher = <PolkadotConfig as Config>::Hasher;
    type Header = <PolkadotConfig as Config>::Header;
    type ExtrinsicParams = DefaultExtrinsicParams<PenpalConfig>;
    type AssetId = <PolkadotConfig as Config>::AssetId;
}

#[derive(Debug)]
pub enum EstimatorError {
    InvalidHexFormat,
    InvalidCommand(String),
    ConnectionError(String),
    /// The value provided does not cover the execution and relayer fee.
    ValueTooLow,
}

impl std::fmt::Display for EstimatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstimatorError::InvalidHexFormat => write!(f, "Command must start with 0x"),
            EstimatorError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            EstimatorError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            EstimatorError::ValueTooLow => write!(
                f,
                "Value provided to low to cover execution and relayer fee"
            ),
        }
    }
}

impl std::error::Error for EstimatorError {}

pub async fn clients() -> Result<Clients, EstimatorError> {
    let asset_hub_client: OnlineClient<AssetHubConfig> =
        OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string())
            .await
            .map_err(|e| {
                EstimatorError::ConnectionError(format!("Cannot connect to asset hub: {}", e))
            })?;

    let bridge_hub_client: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url((*BRIDGE_HUB_WS_URL).to_string())
            .await
            .map_err(|e| {
                EstimatorError::ConnectionError(format!("Cannot connect to bridge hub: {}", e))
            })?;

    let penpal_client: OnlineClient<PenpalConfig> =
        OnlineClient::from_url((*PENPAL_WS_URL).to_string())
            .await
            .map_err(|e| {
                EstimatorError::ConnectionError(format!("Cannot connect to penpal: {}", e))
            })?;

    Ok(Clients {
        asset_hub_client: Box::new(asset_hub_client),
        bridge_hub_client: Box::new(bridge_hub_client),
        penpal_client: Box::new(penpal_client),
    })
}

pub struct Clients {
    pub asset_hub_client: Box<OnlineClient<AssetHubConfig>>,
    pub bridge_hub_client: Box<OnlineClient<PolkadotConfig>>,
    pub penpal_client: Box<OnlineClient<PenpalConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetHubInfo {
    pub execution_fee_in_dot: u128,
    pub execution_fee_in_ether: u128,
    pub delivery_fee_in_dot: u128,
    pub delivery_fee_in_ether: u128,
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DestinationInfo {
    pub execution_fee_in_dot: Option<u128>,
    pub execution_fee_in_ether: Option<u128>,
    pub delivery_fee_in_dot: Option<u128>,
    pub delivery_fee_in_ether: Option<u128>,
    pub dry_run_success: Option<bool>,
    pub dry_run_error: Option<String>,
    pub para_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasEstimation {
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    pub asset_hub: AssetHubInfo,
    pub destination: DestinationInfo,
}

#[derive(Debug)]
pub struct DryRunResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub forwarded_xcm: Option<(VersionedLocation, Vec<VersionedXcm>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BridgeAsset {
    #[serde(rename = "native")]
    NativeToken { token: String, amount: String },
    #[serde(rename = "foreign")]
    ForeignToken { foreign_id: String, amount: String },
}

pub async fn estimate_gas(
    clients: &Clients,
    xcm_bytes: &[u8],
    claimer: Option<Location>,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    relayer_fee: u128,
    assets: &[BridgeAsset],
) -> Result<GasEstimation, EstimatorError> {
    validate(value, execution_fee, relayer_fee)?;
    let claimer_location = get_claimer_location(claimer)?;

    let destination_xcm = build_asset_hub_xcm(
        clients,
        xcm_bytes,
        claimer_location,
        origin,
        value,
        execution_fee,
        relayer_fee,
        assets,
    )
    .await?;

    let weight = query_xcm_weight(clients, destination_xcm.clone()).await?;
    let asset_hub_execution_fee_in_dot = query_weight_to_asset_fee(clients, &weight).await?;

    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };

    let ether_asset = Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
    };

    let asset_hub_execution_fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        asset_hub_execution_fee_in_dot.clone(),
        true,
    )
    .await?;

    let delivery_fee_in_dot = calculate_delivery_fee_in_dot(clients, &destination_xcm).await?;
    let delivery_fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        delivery_fee_in_dot,
        true,
    )
    .await?;

    let extrinsic_fee_in_dot =
        calculate_extrinsic_fee_in_dot(clients, &destination_xcm, origin).await?;
    let extrinsic_fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        extrinsic_fee_in_dot,
        true,
    )
    .await?;

    let ah_dry_run_result = dry_run_xcm_on_asset_hub(clients, &destination_xcm).await?;

    // Run destination dry run if there's a forwarded XCM
    let (
        destination_dry_run_success,
        destination_dry_run_error,
        destination_para_id,
        destination_execution_fee_in_dot,
        destination_execution_fee_in_ether,
        destination_delivery_fee_in_dot,
        destination_delivery_fee_in_ether,
    ) = if ah_dry_run_result.forwarded_xcm.is_some() {
        let forwarded_xcm = ah_dry_run_result.forwarded_xcm.unwrap();

        // Extract destination parachain ID before running dry run
        let para_id = match extract_parachain_id(&forwarded_xcm.0) {
            Ok(id) => Some(id),
            Err(_) => None,
        };

        // Calculate destination execution fee (separate from dry run, like Asset Hub)
        let (dest_fee_dot, dest_fee_ether) = if let Some(para_id_val) = para_id {
            match calculate_destination_execution_fee(
                clients,
                &forwarded_xcm,
                para_id_val,
                &dot_asset,
                &ether_asset,
            )
            .await
            {
                Ok((fee_dot, fee_ether)) => (Some(fee_dot), Some(fee_ether)),
                Err(e) => {
                    println!("Failed to calculate destination execution fee: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        // Calculate AssetHub to destination delivery fee
        let (dest_delivery_fee_dot, dest_delivery_fee_ether) = if let Some(para_id_val) = para_id {
            match calculate_asset_hub_to_destination_delivery_fee(
                clients,
                &forwarded_xcm,
                para_id_val,
            )
            .await
            {
                Ok(delivery_fee_dot) => {
                    // Convert delivery fee to Ether
                    match quote_price_exact_tokens_for_tokens(
                        clients,
                        dot_asset.clone(),
                        ether_asset.clone(),
                        delivery_fee_dot,
                        true,
                    )
                    .await
                    {
                        Ok(delivery_fee_ether) => {
                            (Some(delivery_fee_dot), Some(delivery_fee_ether))
                        }
                        Err(e) => {
                            println!("Failed to convert destination delivery fee to Ether: {}", e);
                            (Some(delivery_fee_dot), None)
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to calculate destination delivery fee: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        match dry_run_xcm_on_destination(clients, forwarded_xcm).await {
            Ok(dest_result) => (
                Some(dest_result.success),
                dest_result.error_message,
                para_id,
                dest_fee_dot,
                dest_fee_ether,
                dest_delivery_fee_dot,
                dest_delivery_fee_ether,
            ),
            Err(e) => (
                Some(false),
                Some(format!("Destination dry run failed: {}", e)),
                para_id,
                dest_fee_dot,
                dest_fee_ether,
                dest_delivery_fee_dot,
                dest_delivery_fee_ether,
            ),
        }
    } else {
        (None, None, None, None, None, None, None)
    };

    Ok(GasEstimation {
        extrinsic_fee_in_dot,
        extrinsic_fee_in_ether,
        asset_hub: AssetHubInfo {
            execution_fee_in_dot: asset_hub_execution_fee_in_dot,
            execution_fee_in_ether: asset_hub_execution_fee_in_ether,
            delivery_fee_in_dot,
            delivery_fee_in_ether,
            dry_run_success: ah_dry_run_result.success,
            dry_run_error: ah_dry_run_result.error_message,
        },
        destination: DestinationInfo {
            dry_run_success: destination_dry_run_success,
            dry_run_error: destination_dry_run_error,
            para_id: destination_para_id,
            execution_fee_in_dot: destination_execution_fee_in_dot,
            execution_fee_in_ether: destination_execution_fee_in_ether,
            delivery_fee_in_dot: destination_delivery_fee_in_dot,
            delivery_fee_in_ether: destination_delivery_fee_in_ether,
        },
    })
}

fn validate(value: u128, execution_fee: u128, relayer_fee: u128) -> Result<(), EstimatorError> {
    if execution_fee.saturating_add(relayer_fee) > value {
        return Err(EstimatorError::ValueTooLow);
    }

    Ok(())
}

pub async fn build_asset_hub_xcm(
    clients: &Clients,
    xcm_bytes: &[u8],
    claimer: Location,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    relayer_fee: u128,
    assets: &[BridgeAsset],
) -> Result<VersionedXcm, EstimatorError> {
    let mut instructions = vec![
        DescendOrigin(X1([PalletInstance(INBOUND_PALLET_V2)])),
        UniversalOrigin(GlobalConsensus(Ethereum { chain_id: CHAIN_ID })),
        ReserveAssetDeposited(Assets(
            vec![
                asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                    id: AssetId(Location {
                        parents: 2,
                        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                    }),
                    fun: Fungible(execution_fee),
                },
            ]
            .into(),
        )),
    ];

    instructions.push(SetHints {
        hints: BoundedVec([AssetClaimer { location: claimer }].into()),
    });

    instructions.push(PayFees {
        asset: asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
            id: AssetId(Location {
                parents: 2,
                interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
            }),
            fun: Fungible(execution_fee),
        },
    });

    let net_value = value.saturating_sub(execution_fee.saturating_add(relayer_fee));
    if net_value > 0 {
        // Asset for remaining ether
        instructions.push(ReserveAssetDeposited(Assets(
            vec![
                asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                    id: AssetId(Location {
                        parents: 2,
                        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                    }),
                    fun: Fungible(net_value),
                },
            ]
            .into(),
        )));
    }

    let mut reserve_deposit_assets = vec![];
    let mut reserve_withdraw_assets = vec![];

    for asset in assets {
        let xcm_asset = convert_asset_to_xcm(clients, asset).await?;
        match asset {
            BridgeAsset::NativeToken { .. } => reserve_deposit_assets.push(xcm_asset),
            BridgeAsset::ForeignToken { .. } => reserve_withdraw_assets.push(xcm_asset),
        }
    }

    if !reserve_deposit_assets.is_empty() {
        instructions.push(ReserveAssetDeposited(Assets(reserve_deposit_assets.into())));
    }

    if !reserve_withdraw_assets.is_empty() {
        instructions.push(WithdrawAsset(Assets(reserve_withdraw_assets.into())));
    }

    instructions.push(DescendOrigin(X1([AccountKey20 {
        key: origin,
        network: None,
    }])));

    let remote_xcm = extract_remote_xcm(xcm_bytes);
    instructions.extend(remote_xcm.0);

    Ok(VersionedXcm::V5(Xcm(instructions)))
}

fn extract_remote_xcm(raw: &[u8]) -> Xcm {
    if let Ok(versioned_xcm) = VersionedXcm::decode_with_depth_limit(8, &mut &raw[..]) {
        if let VersionedXcm::V5(xcm) = versioned_xcm {
            return xcm;
        }
    }
    Xcm(vec![])
}

pub async fn query_xcm_weight(
    clients: &Clients,
    destination_xcm: VersionedXcm,
) -> Result<Weight, EstimatorError> {
    let runtime_api_call = asset_hub_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_xcm_weight(destination_xcm);

    let weight_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to query XCM weight: {:?}", e))
        })?;

    weight_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!("XCM weight query returned error: {:?}", e))
    })
}

pub async fn query_weight_to_asset_fee(
    clients: &Clients,
    weight: &Weight,
) -> Result<u128, EstimatorError> {
    let dot_asset = VersionedAssetId::V5(AssetId(Location {
        parents: 1,
        interior: Here,
    }));

    let runtime_api_call = asset_hub_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_weight_to_asset_fee(weight.clone(), dot_asset);

    let fee_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to query weight to asset fee: {:?}", e))
        })?;

    fee_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!("Weight to asset fee query returned error: {:?}", e))
    })
}

pub async fn quote_price_exact_tokens_for_tokens(
    clients: &Clients,
    asset1: Location,
    asset2: Location,
    asset1_balance: u128,
    include_fee: bool,
) -> Result<u128, EstimatorError> {
    let runtime_api_call = asset_hub_westend_local_runtime::runtime::apis()
        .asset_conversion_api()
        .quote_price_exact_tokens_for_tokens(asset1, asset2, asset1_balance, include_fee);

    let quote_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to quote price for tokens: {:?}", e))
        })?;

    quote_result.ok_or_else(|| {
        EstimatorError::InvalidCommand("Quote price query returned None".to_string())
    })
}

async fn calculate_delivery_fee_in_dot(
    clients: &Clients,
    xcm: &VersionedXcm,
) -> Result<u128, EstimatorError> {
    let destination = bridge_hub_westend_local_runtime::runtime_types::staging_xcm::v5::location::Location {
        parents: 1,
        interior: bridge_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1([
            bridge_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(ASSET_HUB_PARA_ID)
        ]),
    };

    // Convert XCM to bridge hub types for the query
    let encoded_xcm = codec::Encode::encode(xcm);
    let bridge_hub_xcm: bridge_hub_westend_local_runtime::runtime_types::xcm::VersionedXcm =
        codec::Decode::decode(&mut &encoded_xcm[..]).map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to convert XCM types: {:?}", e))
        })?;

    let versioned_destination =
        bridge_hub_westend_local_runtime::runtime_types::xcm::VersionedLocation::V5(destination);

    // Query delivery fees using XCM Payment API
    let runtime_api_call = bridge_hub_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_delivery_fees(versioned_destination, bridge_hub_xcm);

    let fees_result = clients
        .bridge_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to query delivery fees: {:?}", e))
        })?;

    let fees = fees_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!("Delivery fees query returned error: {:?}", e))
    })?;

    // Find DOT asset in the result (parents: 1, interior: Here)
    let assets = match fees {
        bridge_hub_westend_local_runtime::runtime_types::xcm::VersionedAssets::V5(assets) => assets,
        _ => {
            return Err(EstimatorError::InvalidCommand(
                "Unsupported VersionedAssets version".to_string(),
            ))
        }
    };

    for asset in assets.0.iter() {
        if asset.id.0.parents == 1 && matches!(asset.id.0.interior, bridge_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::Here) {
            if let bridge_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Fungibility::Fungible(amount) = asset.fun {
                return Ok(amount);
            }
        }
    }

    Err(EstimatorError::InvalidCommand(
        "Could not find DOT asset in delivery fees result".to_string(),
    ))
}

async fn calculate_asset_hub_to_destination_delivery_fee(
    clients: &Clients,
    forwarded_xcm: &(VersionedLocation, Vec<VersionedXcm>),
    destination_para_id: u32,
) -> Result<u128, EstimatorError> {
    let (destination_location, xcms) = forwarded_xcm;

    // Get the first XCM that will be sent to the destination
    let xcm = xcms.iter().next().ok_or_else(|| {
        EstimatorError::InvalidCommand("No XCM to calculate delivery fee for".to_string())
    })?;

    // Convert XCM to asset hub types for the query
    let encoded_xcm = codec::Encode::encode(xcm);
    let asset_hub_xcm: VersionedXcm =
        codec::Decode::decode(&mut &encoded_xcm[..]).map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to convert XCM types: {:?}", e))
        })?;

    // Convert destination location to asset hub types
    let encoded_destination = codec::Encode::encode(destination_location);
    let asset_hub_destination: VersionedLocation =
        codec::Decode::decode(&mut &encoded_destination[..]).map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to convert destination location types: {:?}",
                e
            ))
        })?;

    // Query delivery fees using AssetHub's XCM Payment API
    let runtime_api_call = asset_hub_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_delivery_fees(asset_hub_destination, asset_hub_xcm);

    let fees_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to query AssetHub delivery fees to para {}: {:?}",
                destination_para_id, e
            ))
        })?;

    let fees = fees_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!(
            "AssetHub delivery fees query returned error: {:?}",
            e
        ))
    })?;

    // Find DOT asset in the result (parents: 1, interior: Here)
    let assets = match fees {
        VersionedAssets::V5(assets) => assets,
        _ => {
            return Err(EstimatorError::InvalidCommand(
                "Unsupported VersionedAssets version for AssetHub delivery fees".to_string(),
            ))
        }
    };

    for asset in assets.0.iter() {
        if asset.id.0.parents == 1 && matches!(asset.id.0.interior, Here) {
            if let Fungible(amount) = asset.fun {
                return Ok(amount);
            }
        }
    }

    Err(EstimatorError::InvalidCommand(format!(
        "Could not find DOT asset in AssetHub delivery fees result for para {}",
        destination_para_id
    )))
}

async fn dry_run_xcm_on_asset_hub(
    clients: &Clients,
    xcm: &VersionedXcm,
) -> Result<DryRunResult, EstimatorError> {
    let bridge_hub_location = asset_hub_westend_local_runtime::runtime_types::xcm::VersionedLocation::V5(
        Location {
            parents: 1,
            interior: X1([
                asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(BRIDGE_HUB_PARA_ID)
            ]),
        }
    );

    let runtime_api_call = asset_hub_westend_local_runtime::runtime::apis()
        .dry_run_api()
        .dry_run_xcm(bridge_hub_location, xcm.clone());

    let dry_run_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to dry run XCM: {:?}", e)))?;

    match dry_run_result {
        Ok(effects) => {
            let success = matches!(effects.execution_result, Outcome::Complete { .. });

            let error_message = if success {
                None
            } else {
                Some(format!(
                    "XCM execution failed: {:?}",
                    effects.execution_result
                ))
            };

            let forwarded_xcms = effects.forwarded_xcms;

            let forwarded_xcm = if forwarded_xcms.is_empty() {
                None
            } else {
                Some(forwarded_xcms[0].clone())
            };

            Ok(DryRunResult {
                success,
                error_message,
                forwarded_xcm,
            })
        }
        Err(e) => Ok(DryRunResult {
            success: false,
            error_message: Some(format!("Dry run API error: {:?}", e)),
            forwarded_xcm: None,
        }),
    }
}

#[cfg(feature = "local")]
async fn dry_run_xcm_on_destination(
    clients: &Clients,
    forwarded_xcm: (VersionedLocation, Vec<VersionedXcm>),
) -> Result<DryRunResult, EstimatorError> {
    let (destination_location, xcms) = forwarded_xcm;

    let para_id = extract_parachain_id(&destination_location)?;

    let xcm = xcms
        .into_iter()
        .next()
        .ok_or_else(|| EstimatorError::InvalidCommand("No XCM to forward".to_string()))?;

    match para_id {
        PENPAL_PARA_ID => penpal::dry_run_xcm(clients, xcm).await,
        _ => Err(EstimatorError::InvalidCommand(format!(
            "Unsupported destination parachain for local network: {}",
            para_id
        ))),
    }
}

#[cfg(feature = "local")]
async fn calculate_destination_execution_fee(
    clients: &Clients,
    forwarded_xcm: &(VersionedLocation, Vec<VersionedXcm>),
    para_id: u32,
    dot_asset: &Location,
    ether_asset: &Location,
) -> Result<(u128, u128), EstimatorError> {
    let (_, xcms) = forwarded_xcm;
    let xcm = xcms
        .iter()
        .next()
        .ok_or_else(|| EstimatorError::InvalidCommand("No XCM to calculate fee for".to_string()))?;

    match para_id {
        PENPAL_PARA_ID => {
            let fee_dot = crate::penpal::calculate_execution_fee(clients, xcm).await?;
            let fee_ether = quote_price_exact_tokens_for_tokens(
                clients,
                dot_asset.clone(),
                ether_asset.clone(),
                fee_dot,
                true,
            )
            .await?;

            Ok((fee_dot, fee_ether))
        }
        _ => Err(EstimatorError::InvalidCommand(format!(
            "Fee calculation not supported for parachain {} on local network",
            para_id
        ))),
    }
}

fn extract_parachain_id(location: &VersionedLocation) -> Result<u32, EstimatorError> {
    match location {
        VersionedLocation::V5(loc) => {
            match &loc.interior {
                asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1(junction_array) => {
                    match &junction_array[0] {
                        asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(para_id) => {
                            Ok(*para_id)
                        },
                        _ => Err(EstimatorError::InvalidCommand("Location does not contain parachain junction".to_string()))
                    }
                },
                _ => Err(EstimatorError::InvalidCommand("Unsupported location format".to_string()))
            }
        },
        _ => Err(EstimatorError::InvalidCommand("Unsupported location version".to_string()))
    }
}

async fn calculate_extrinsic_fee_in_dot(
    clients: &Clients,
    _xcm: &VersionedXcm,
    _origin: [u8; 20],
) -> Result<u128, EstimatorError> {
    use bridge_hub_westend_local_runtime::runtime_types::snowbridge_verification_primitives::EventProof;
    let fixture = snowbridge_pallet_ethereum_client_fixtures::make_inbound_fixture();
    let encoded_event_proof = codec::Encode::encode(&fixture.event);

    let runtime_event_proof: EventProof = codec::Decode::decode(&mut &encoded_event_proof[..])
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to decode EventProof: {:?}", e))
        })?;

    let submit_call = bridge_hub_westend_local_runtime::runtime::tx()
        .ethereum_inbound_queue_v2()
        .submit(runtime_event_proof);

    let alice = dev::alice();

    let fee = clients
        .bridge_hub_client
        .tx()
        .create_signed(&submit_call, &alice, Default::default())
        .await
        .unwrap()
        .partial_fee_estimate()
        .await
        .unwrap();

    Ok(fee)
}

async fn lookup_foreign_asset_location(
    clients: &Clients,
    foreign_id: &str,
) -> Result<Location, EstimatorError> {
    let foreign_id_bytes = parse_hex_string(foreign_id)?;

    if foreign_id_bytes.len() != 32 {
        return Err(EstimatorError::InvalidCommand(format!(
            "Foreign ID must be 32 bytes, got {}",
            foreign_id_bytes.len()
        )));
    }

    let h256 = H256::from_slice(&foreign_id_bytes);

    let storage_query = bridge_hub_westend_local_runtime::runtime::storage()
        .ethereum_system()
        .foreign_to_native_id(h256);

    let bridge_location_result = clients
        .bridge_hub_client
        .storage()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .fetch(&storage_query)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to query foreign asset location: {:?}",
                e
            ))
        })?;

    let bridge_location = bridge_location_result.ok_or_else(|| {
        EstimatorError::InvalidCommand(format!("Foreign asset ID not found: {}", foreign_id))
    })?;

    // Convert from bridge hub Location to asset hub Location
    let encoded = codec::Encode::encode(&bridge_location);
    let decoded_location: Location = codec::Decode::decode(&mut &encoded[..]).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to convert Location types: {:?}", e))
    })?;

    Ok(decoded_location)
}

pub fn decode_assets(assets_json: &str) -> Result<Vec<BridgeAsset>, EstimatorError> {
    let assets: Vec<BridgeAsset> = serde_json::from_str(assets_json).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to parse assets JSON: {}", e))
    })?;

    Ok(assets)
}

fn parse_amount_string(amount_str: &str) -> Result<u128, EstimatorError> {
    amount_str
        .parse::<u128>()
        .map_err(|e| EstimatorError::InvalidCommand(format!("Invalid amount: {}", e)))
}

fn parse_hex_string(hex_str: &str) -> Result<Vec<u8>, EstimatorError> {
    let hex_str = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };

    hex::decode(hex_str)
        .map_err(|e| EstimatorError::InvalidCommand(format!("Invalid hex string: {}", e)))
}

async fn convert_asset_to_xcm(
    clients: &Clients,
    asset: &BridgeAsset,
) -> Result<
    asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset,
    EstimatorError,
> {
    match asset {
        BridgeAsset::NativeToken { token, amount } => {
            let amount_value = parse_amount_string(amount)?;
            let token_bytes = parse_hex_string(token)?;

            if token_bytes.len() != 20 {
                return Err(EstimatorError::InvalidCommand(format!(
                    "Token address must be 20 bytes, got {}",
                    token_bytes.len()
                )));
            }

            let mut address_bytes = [0u8; 20];
            address_bytes.copy_from_slice(&token_bytes);

            Ok(asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X2([
                        GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID }),
                        asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::junction::Junction::AccountKey20 {
                            network: None,
                            key: address_bytes,
                        }
                    ]),
                }),
                fun: Fungible(amount_value),
            })
        }
        BridgeAsset::ForeignToken { foreign_id, amount } => {
            let amount_value = parse_amount_string(amount)?;
            let location = lookup_foreign_asset_location(clients, foreign_id).await?;

            Ok(
                asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                    id: AssetId(location),
                    fun: Fungible(amount_value),
                },
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum Network {
    Polkadot = 0,
}

impl Network {
    pub fn from_u8(value: u8) -> Result<Self, EstimatorError> {
        match value {
            0 => Ok(Network::Polkadot),
            _ => Err(EstimatorError::InvalidCommand(format!(
                "Unsupported network: {}. Only network 0 (Polkadot) is supported",
                value
            ))),
        }
    }
}

/// Get sovereign account of Ethereum on Asset Hub.
fn bridge_owner() -> Result<RuntimeAccountId32, EstimatorError> {
    use xcm::opaque::latest::InteriorLocation;
    use xcm::opaque::latest::WESTEND_GENESIS_HASH;
    use xcm::prelude::*;
    use xcm_builder::ExternalConsensusLocationsConverterFor;
    use xcm_executor::traits::ConvertLocation;
    parameter_types! {
        pub AssetHubUniversalLocation: InteriorLocation = [GlobalConsensus(NetworkId::ByGenesis(WESTEND_GENESIS_HASH)), Parachain(ASSET_HUB_PARA_ID)].into();
    }

    let bridge_owner = ExternalConsensusLocationsConverterFor::<
        AssetHubUniversalLocation,
        [u8; 32],
    >::convert_location(&Location::new(
        2,
        [GlobalConsensus(Ethereum{ chain_id: CHAIN_ID })],
    ))
        .unwrap();

    Ok(bridge_owner.into())
}

/// Construct the remote XCM needed to create a new asset in the `ForeignAssets` pallet
/// on AssetHub. Polkadot is the only supported network at the moment.
pub fn construct_register_token_xcm(
    token_address_hex: &str,
    network: u8,
    eth_value: u128,
    claimer: Option<Location>,
) -> Result<Vec<u8>, EstimatorError> {
    let claimer_location = get_claimer_location(claimer)?;
    let network = Network::from_u8(network)?;

    // Parse token address
    let token_bytes =
        hex::decode(&token_address_hex[2..]).map_err(|_| EstimatorError::InvalidHexFormat)?;
    if token_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(
            "Token address must be 20 bytes (H160)".to_string(),
        ));
    }
    let mut token = [0u8; 20];
    token.copy_from_slice(&token_bytes);

    let xcm = make_create_asset_xcm(&token, network, eth_value, claimer_location)?;
    let versioned_xcm = VersionedXcm::V5(xcm);
    let xcm_bytes = codec::Encode::encode(&versioned_xcm);
    Ok(xcm_bytes)
}

/// Construct the remote XCM needed to create a new asset in the `ForeignAssets` pallet
/// on AssetHub. Polkadot is the only supported network at the moment.
fn make_create_asset_xcm(
    token: &[u8; 20],
    network: Network,
    eth_value: u128,
    claimer: Location,
) -> Result<Xcm, EstimatorError> {
    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };
    let dot_fee_asset =
        asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
            id: AssetId(dot_asset),
            fun: Fungible(CREATE_ASSET_DEPOSIT),
        };

    let eth_asset = asset_hub_westend_local_runtime::runtime_types::staging_xcm::v5::asset::Asset {
        id: AssetId(Location {
            parents: 2,
            interior: X1([GlobalConsensus(Ethereum { chain_id: CHAIN_ID })]),
        }),
        fun: Fungible(eth_value),
    };

    let asset_id = Location {
        parents: 2,
        interior: X2([
            GlobalConsensus(Ethereum { chain_id: CHAIN_ID }),
            AccountKey20 {
                network: None,
                key: *token,
            },
        ]),
    };

    let bridge_owner = bridge_owner()?;

    match network {
        Network::Polkadot => Ok(make_create_asset_xcm_for_polkadot(
            CREATE_ASSET_CALL,
            asset_id,
            dot_fee_asset,
            eth_asset,
            claimer,
            bridge_owner,
        )),
    }
}

/// Construct the asset creation XCM for the Polkadot network.
fn make_create_asset_xcm_for_polkadot(
    create_call_index: [u8; 2],
    asset_id: Location,
    dot_fee_asset: Asset,
    eth_asset: Asset,
    claimer: Location,
    bridge_owner: RuntimeAccountId32,
) -> Xcm {
    let bridge_owner_bytes: [u8; 32] = bridge_owner.into();
    use sp_runtime::MultiAddress;
    Xcm(vec![
        ExchangeAsset {
            give: Definite(Assets(vec![eth_asset])),
            want: Assets(vec![dot_fee_asset.clone()]),
            maximal: false,
        },
        DepositAsset {
            assets: Definite(Assets(vec![dot_fee_asset.clone()].into())),
            beneficiary: Location {
                parents: 0,
                interior: X1([AccountId32 {
                    network: None,
                    id: bridge_owner_bytes,
                }]),
            },
        },
        Transact {
            origin_kind: OriginKind::Xcm,
            fallback_max_weight: None,
            call:
                asset_hub_westend_local_runtime::runtime_types::xcm::double_encoded::DoubleEncoded {
                    encoded: codec::Encode::encode(&(
                        create_call_index,
                        asset_id.clone(),
                        MultiAddress::<[u8; 32], ()>::Id(bridge_owner_bytes.into()),
                        MINIMUM_DEPOSIT,
                    )),
                },
        },
        RefundSurplus,
        DepositAsset {
            assets: Wild(AllCounted(2)).into(),
            beneficiary: claimer,
        },
    ])
}

fn get_claimer_location(claimer: Option<Location>) -> Result<Location, EstimatorError> {
   match claimer {
        Some(loc) => Ok(loc),
        None => Ok(Location {
            parents: 0,
            interior: X1([AccountId32 { network: None, id: bridge_owner()?.into() }]),
        })
    }
}
