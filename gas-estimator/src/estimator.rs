use asset_hub_westend_runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use asset_hub_westend_runtime::runtime_types::sp_weights::weight_v2::Weight;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Fungibility::Fungible;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::{AssetId, Assets};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::{
    AccountKey20, GlobalConsensus, PalletInstance,
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId::Ethereum;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::Here;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::{X1, X2};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::traits::Outcome;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Hint::AssetClaimer;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::UniversalOrigin;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::{
    DescendOrigin, PayFees, ReserveAssetDeposited, SetHints, WithdrawAsset,
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Xcm;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedAssetId;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedXcm;
use codec::DecodeLimit;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use std::env;
use subxt::{config::DefaultExtrinsicParams, Config, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

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

    pub static ref PENPAL_HUB_WS_URL: String = {
        if let Ok(val) = env::var("PENPAL_HUB_WS_URL") {
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
            EstimatorError::ValueTooLow => write!(f, "Value provided to low to cover execution and relayer fee"),
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

    Ok(Clients {
        asset_hub_client: Box::new(asset_hub_client),
        bridge_hub_client: Box::new(bridge_hub_client),
    })
}

pub struct Clients {
    pub asset_hub_client: Box<OnlineClient<AssetHubConfig>>,
    pub bridge_hub_client: Box<OnlineClient<PolkadotConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasEstimation {
    pub fee_in_dot: u128,
    pub fee_in_ether: u128,
    pub delivery_fee_in_dot: u128,
    pub delivery_fee_in_ether: u128,
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    pub asset_hub_xcm: String,
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DryRunResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub forwarded_destination: Option<String>,
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
    claimer: Location,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    relayer_fee: u128,
    assets: &[BridgeAsset],
) -> Result<GasEstimation, EstimatorError> {
    validate(value, execution_fee, relayer_fee)?;

    let destination_xcm =
        build_asset_hub_xcm(clients, xcm_bytes, claimer, origin, value, execution_fee, relayer_fee, assets).await?;

    let asset_hub_xcm = format!("{:?}", destination_xcm);

    let _weight = query_xcm_weight(clients, destination_xcm.clone()).await?;
    let fee_in_dot = query_weight_to_asset_fee(clients, &_weight).await?;

    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };

    let ether_asset = Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
    };

    let fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        fee_in_dot.clone(),
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

    let dry_run_result = dry_run_xcm_on_asset_hub(clients, &destination_xcm).await?;

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

    Ok(GasEstimation {
        fee_in_dot,
        fee_in_ether,
        delivery_fee_in_dot,
        delivery_fee_in_ether,
        extrinsic_fee_in_dot,
        extrinsic_fee_in_ether,
        asset_hub_xcm,
        dry_run_success: dry_run_result.success,
        dry_run_error: dry_run_result.error_message,
    })
}

fn validate(value: u128, execution_fee: u128, relayer_fee: u128) -> Result<(), EstimatorError> {
    if execution_fee.saturating_add(relayer_fee) > value {
        return Err(EstimatorError::ValueTooLow)
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
                asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset {
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
        asset: asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset {
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
        instructions.push(ReserveAssetDeposited(Assets(vec![asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset {
            id: AssetId(Location {
                parents: 2,
                interior: X1([
                    GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID }),
                ]),
            }),
            fun: Fungible(net_value),
        }].into())));
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
    let runtime_api_call = asset_hub_westend_runtime::runtime::apis()
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

    let runtime_api_call = asset_hub_westend_runtime::runtime::apis()
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
    let runtime_api_call = asset_hub_westend_runtime::runtime::apis()
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
    let destination = bridge_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location {
        parents: 1,
        interior: bridge_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1([
            bridge_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(ASSET_HUB_PARA_ID)
        ]),
    };

    // Convert XCM to bridge hub types for the query
    let encoded_xcm = codec::Encode::encode(xcm);
    let bridge_hub_xcm: bridge_hub_westend_runtime::runtime_types::xcm::VersionedXcm =
        codec::Decode::decode(&mut &encoded_xcm[..]).map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to convert XCM types: {:?}", e))
        })?;

    let versioned_destination =
        bridge_hub_westend_runtime::runtime_types::xcm::VersionedLocation::V5(destination);

    // Query delivery fees using XCM Payment API
    let runtime_api_call = bridge_hub_westend_runtime::runtime::apis()
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
        bridge_hub_westend_runtime::runtime_types::xcm::VersionedAssets::V5(assets) => assets,
        _ => {
            return Err(EstimatorError::InvalidCommand(
                "Unsupported VersionedAssets version".to_string(),
            ))
        }
    };

    for asset in assets.0.iter() {
        if asset.id.0.parents == 1 && matches!(asset.id.0.interior, bridge_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::Here) {
            if let bridge_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Fungibility::Fungible(amount) = asset.fun {
                return Ok(amount);
            }
        }
    }

    Err(EstimatorError::InvalidCommand(
        "Could not find DOT asset in delivery fees result".to_string(),
    ))
}

async fn dry_run_xcm_on_asset_hub(
    clients: &Clients,
    xcm: &VersionedXcm,
) -> Result<DryRunResult, EstimatorError> {
    // Create bridge hub location (parents: 1, interior: X1([Parachain(1002)]))
    let bridge_hub_location = asset_hub_westend_runtime::runtime_types::xcm::VersionedLocation::V5(
        Location {
            parents: 1,
            interior: X1([
                asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(BRIDGE_HUB_PARA_ID)
            ]),
        }
    );

    // Query dry run XCM using Asset Hub's dry run API
    let runtime_api_call = asset_hub_westend_runtime::runtime::apis()
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

            Ok(DryRunResult {
                success,
                error_message,
                forwarded_destination: None, // TODO: Parse forwarded XCMs
            })
        }
        Err(e) => Ok(DryRunResult {
            success: false,
            error_message: Some(format!("Dry run API error: {:?}", e)),
            forwarded_destination: None,
        }),
    }
}

async fn calculate_extrinsic_fee_in_dot(
    clients: &Clients,
    _xcm: &VersionedXcm,
    _origin: [u8; 20],
) -> Result<u128, EstimatorError> {
    use bridge_hub_westend_runtime::runtime_types::snowbridge_verification_primitives::EventProof;
    let fixture = snowbridge_pallet_ethereum_client_fixtures::make_inbound_fixture();
    let encoded_event_proof = codec::Encode::encode(&fixture.event);

    let runtime_event_proof: EventProof = codec::Decode::decode(&mut &encoded_event_proof[..])
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to decode EventProof: {:?}", e))
        })?;

    let submit_call = bridge_hub_westend_runtime::runtime::tx()
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

    let storage_query = bridge_hub_westend_runtime::runtime::storage()
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
) -> Result<asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset, EstimatorError>
{
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

            Ok(asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X2([
                        GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID }),
                        asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::AccountKey20 {
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
                asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Asset {
                    id: AssetId(location),
                    fun: Fungible(amount_value),
                },
            )
        }
    }
}
