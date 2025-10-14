#[cfg(feature = "local")]
use crate::config::local::*;
use crate::contracts::r#i_gateway_v2::IGatewayV2;
use alloy_sol_types::{sol, SolValue};
#[cfg(feature = "local")]
use asset_hub_westend_local_runtime::{
    runtime as asset_hub_runtime,
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        staging_xcm::v5::{
            asset::{
                Asset,
                AssetFilter::{Definite, Wild},
                AssetId, Assets,
                Fungibility::Fungible,
                WildAsset::AllCounted,
            },
            junction::{
                Junction::{
                    AccountId32, AccountKey20, GlobalConsensus, PalletInstance,
                },
                NetworkId::{self, Ethereum},
            },
            junctions::{
                Junctions::{Here, X1, X2},
            },
            location::Location,
            Hint::AssetClaimer,
            Instruction::{
                DepositAsset, DescendOrigin, ExchangeAsset, PayFees, RefundSurplus,
                ReserveAssetDeposited, SetHints, Transact, UniversalOrigin, WithdrawAsset,
            },
            Xcm,
        },
        xcm::{
            double_encoded::DoubleEncoded, v3::OriginKind, VersionedXcm,
        },
    },
};
#[cfg(feature = "local")]
use bridge_hub_westend_local_runtime::{
    runtime as bridge_hub_runtime,
    runtime_types::{
        snowbridge_verification_primitives::EventProof,
        staging_xcm::v5::{
            asset::Fungibility as BridgeHubFungibility,
            junction::Junction::Parachain as BridgeHubParachain,
            junctions::Junctions as BridgeHubJunctions,
            location::Location as BridgeHubLocation,
        },
        xcm::{
            VersionedAssets as BridgeHubVersionedAssets,
            VersionedLocation as BridgeHubVersionedLocation, VersionedXcm as BridgeHubVersionedXcm,
        },
    },
};
use codec::DecodeLimit;
use hex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sp_core::{parameter_types, H256};
use sp_runtime::AccountId32 as RuntimeAccountId32;
use std::env;
use subxt::{config::DefaultExtrinsicParams, Config, OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

sol! {
    struct AsNativeTokenERC20 {
        address token;
        uint128 amount;
    }

    struct AsForeignTokenERC20 {
        bytes32 foreignID;
        uint128 amount;
    }

    struct AsCreateAsset {
        address token;
        uint8 network;
    }
}

lazy_static! {
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
}

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
pub struct AssetHubInfo {
    pub delivery_fee_in_dot: u128,
    pub delivery_fee_in_ether: u128,
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasEstimation {
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    pub asset_hub: AssetHubInfo,
}

#[derive(Debug)]
pub struct DryRunResult {
    pub success: bool,
    pub error_message: Option<String>,
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
    event_log_address: &str,
    event_log_topics: &str,
    event_log_data: &str,
    proof_hex: &str,
    xcm_bytes: &[u8],
    claimer: Option<Location>,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    _relayer_fee: u128,
    assets: &[BridgeAsset],
) -> Result<GasEstimation, EstimatorError> {
    // Construct EventProof from the provided parameters
    let event_proof = construct_event_proof(
        event_log_address,
        event_log_topics,
        event_log_data,
        proof_hex,
    )?;

    // Calculate extrinsic fee for submitting to BridgeHub using the actual EventProof
    let extrinsic_fee_in_dot = calculate_extrinsic_fee_in_dot(clients, &event_proof).await?;

    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };

    let ether_asset = Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
    };

    let extrinsic_fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        extrinsic_fee_in_dot,
        true,
    )
    .await?;

    // Build AssetHub XCM for delivery fee calculation
    let claimer_location = get_claimer_location(claimer)?;
    let destination_xcm = build_asset_hub_xcm(
        clients,
        xcm_bytes,
        claimer_location,
        origin,
        value,
        execution_fee,
        assets,
    )
    .await?;

    // Calculate delivery fee based on the XCM
    let delivery_fee_in_dot = calculate_delivery_fee_in_dot(clients, &destination_xcm).await?;
    let delivery_fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset.clone(),
        ether_asset.clone(),
        delivery_fee_in_dot,
        true,
    )
    .await?;

    // Perform dry-run of the submit extrinsic on BridgeHub using the actual EventProof
    let dry_run_result = dry_run_submit_on_bridge_hub(clients, &event_proof).await?;

    Ok(GasEstimation {
        extrinsic_fee_in_dot,
        extrinsic_fee_in_ether,
        asset_hub: AssetHubInfo {
            delivery_fee_in_dot,
            delivery_fee_in_ether,
            dry_run_success: dry_run_result.success,
            dry_run_error: dry_run_result.error_message,
        },
    })
}

pub async fn build_asset_hub_xcm(
    clients: &Clients,
    xcm_bytes: &[u8],
    claimer: Location,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    assets: &[BridgeAsset],
) -> Result<VersionedXcm, EstimatorError> {
    let mut instructions = vec![
        DescendOrigin(X1([PalletInstance(INBOUND_PALLET_V2)])),
        UniversalOrigin(GlobalConsensus(Ethereum { chain_id: CHAIN_ID })),
        ReserveAssetDeposited(Assets(
            vec![Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                }),
                fun: Fungible(execution_fee),
            }]
            .into(),
        )),
        SetHints {
            hints: BoundedVec([AssetClaimer { location: claimer }].into()),
        },
        PayFees {
            asset: Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                }),
                fun: Fungible(execution_fee),
            },
        },
    ];

    if value > 0 {
        // Asset for remaining ether
        instructions.push(ReserveAssetDeposited(Assets(
            vec![Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                }),
                fun: Fungible(value),
            }]
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

pub async fn quote_price_exact_tokens_for_tokens(
    clients: &Clients,
    asset1: Location,
    asset2: Location,
    asset1_balance: u128,
    include_fee: bool,
) -> Result<u128, EstimatorError> {
    let runtime_api_call = asset_hub_runtime::apis()
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
    let destination = BridgeHubLocation {
        parents: 1,
        interior: BridgeHubJunctions::X1([BridgeHubParachain(ASSET_HUB_PARA_ID)]),
    };

    // Convert XCM to bridge hub types for the query
    let encoded_xcm = codec::Encode::encode(xcm);
    let bridge_hub_xcm: BridgeHubVersionedXcm = codec::Decode::decode(&mut &encoded_xcm[..])
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to convert XCM types: {:?}", e))
        })?;

    let versioned_destination = BridgeHubVersionedLocation::V5(destination);

    // Query delivery fees using XCM Payment API
    let runtime_api_call = bridge_hub_runtime::apis()
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
        BridgeHubVersionedAssets::V5(assets) => assets,
        _ => {
            return Err(EstimatorError::InvalidCommand(
                "Unsupported VersionedAssets version".to_string(),
            ))
        }
    };

    for asset in assets.0.iter() {
        if asset.id.0.parents == 1 && matches!(asset.id.0.interior, BridgeHubJunctions::Here) {
            if let BridgeHubFungibility::Fungible(amount) = asset.fun {
                return Ok(amount);
            }
        }
    }

    Err(EstimatorError::InvalidCommand(
        "Could not find DOT asset in delivery fees result".to_string(),
    ))
}

fn construct_event_proof(
    event_log_address: &str,
    event_log_topics: &str,
    event_log_data: &str,
    proof_hex: &str,
) -> Result<EventProof, EstimatorError> {
    use bridge_hub_westend_local_runtime::runtime_types::{
        snowbridge_verification_primitives::{
            Log,
            Proof,
        },
    };

    use sp_core::{H160, H256};

    let address_bytes = parse_hex_string(event_log_address)?;
    if address_bytes.len() != 20 {
        return Err(EstimatorError::InvalidCommand(format!(
            "Event log address must be 20 bytes, got {}",
            address_bytes.len()
        )));
    }
    let address = H160::from_slice(&address_bytes);

    let topics: Result<Vec<H256>, EstimatorError> = event_log_topics
        .split(',')
        .map(|topic_str| {
            let topic_bytes = parse_hex_string(topic_str.trim())?;
            if topic_bytes.len() != 32 {
                return Err(EstimatorError::InvalidCommand(format!(
                    "Each topic must be 32 bytes, got {}",
                    topic_bytes.len()
                )));
            }
            Ok(H256::from_slice(&topic_bytes))
        })
        .collect();
    let topics = topics?;

    let data = parse_hex_string(event_log_data)?;

    let proof_bytes = parse_hex_string(proof_hex)?;
    let proof: Proof = codec::Decode::decode(&mut &proof_bytes[..])
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to decode proof: {:?}", e))
        })?;

    let log = Log {
        address,
        topics,
        data: data.into(),
    };

    Ok(EventProof {
        event_log: log,
        proof,
    })
}

async fn dry_run_submit_on_bridge_hub(
    clients: &Clients,
    event_proof: &EventProof,
) -> Result<DryRunResult, EstimatorError> {
    let submit_call = bridge_hub_runtime::tx()
        .ethereum_inbound_queue_v2()
        .submit(event_proof.clone());

    let alice = dev::alice();

    let _signed = clients
        .bridge_hub_client
        .tx()
        .create_signed(&submit_call, &alice, Default::default())
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to create signed call: {:?}", e))
        })?;

    Ok(DryRunResult {
        success: true,
        error_message: None,
    })
}

async fn calculate_extrinsic_fee_in_dot(
    clients: &Clients,
    event_proof: &EventProof,
) -> Result<u128, EstimatorError> {
    let submit_call = bridge_hub_runtime::tx()
        .ethereum_inbound_queue_v2()
        .submit(event_proof.clone());

    let alice = dev::alice();

    let fee = clients
        .bridge_hub_client
        .tx()
        .create_signed(&submit_call, &alice, Default::default())
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to create signed transaction: {:?}", e))
        })?
        .partial_fee_estimate()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to estimate fee: {:?}", e))
        })?;

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

    let storage_query = bridge_hub_runtime::storage()
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

pub fn decode_assets_from_hex(assets_hex: &str) -> Result<Vec<BridgeAsset>, EstimatorError> {
    if assets_hex == "" || assets_hex.is_empty() {
        return Ok(vec![]);
    }

    let hex_str = if assets_hex.starts_with("0x") {
        &assets_hex[2..]
    } else {
        assets_hex
    };

    let data = hex::decode(hex_str).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to decode hex assets: {}", e))
    })?;

    let assets_vec = Vec::<IGatewayV2::Asset>::abi_decode(&data).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to ABI decode assets: {}", e))
    })?;

    let mut bridge_assets = Vec::new();
    for (i, asset) in assets_vec.iter().enumerate() {
        let bridge_asset = match asset.kind {
            0 => {
                // Native token - decode the inner data
                let decoded = AsNativeTokenERC20::abi_decode(&asset.data).map_err(|e| {
                    EstimatorError::InvalidCommand(format!(
                        "Failed to decode native token data for asset {}: {}",
                        i, e
                    ))
                })?;

                BridgeAsset::NativeToken {
                    token: format!("{:?}", decoded.token),
                    amount: decoded.amount.to_string(),
                }
            }
            1 => {
                // Foreign token - decode the inner data
                let decoded = AsForeignTokenERC20::abi_decode(&asset.data).map_err(|e| {
                    EstimatorError::InvalidCommand(format!(
                        "Failed to decode foreign token data for asset {}: {}",
                        i, e
                    ))
                })?;

                BridgeAsset::ForeignToken {
                    foreign_id: format!("0x{}", hex::encode(decoded.foreignID)),
                    amount: decoded.amount.to_string(),
                }
            }
            _ => {
                return Err(EstimatorError::InvalidCommand(format!(
                    "Unknown asset kind {} for asset {}",
                    asset.kind, i
                )));
            }
        };

        bridge_assets.push(bridge_asset);
    }

    Ok(bridge_assets)
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
) -> Result<Asset, EstimatorError> {
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

            Ok(Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X2([
                        GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID }),
                        AccountKey20 {
                            network: None,
                            key: address_bytes,
                        },
                    ]),
                }),
                fun: Fungible(amount_value),
            })
        }
        BridgeAsset::ForeignToken { foreign_id, amount } => {
            let amount_value = parse_amount_string(amount)?;
            let location = lookup_foreign_asset_location(clients, foreign_id).await?;

            Ok(Asset {
                id: AssetId(location),
                fun: Fungible(amount_value),
            })
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
    let dot_fee_asset = Asset {
        id: AssetId(dot_asset),
        fun: Fungible(CREATE_ASSET_DEPOSIT),
    };

    let eth_asset = Asset {
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
            call: DoubleEncoded {
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
            interior: X1([AccountId32 {
                network: None,
                id: bridge_owner()?.into(),
            }]),
        }),
    }
}
