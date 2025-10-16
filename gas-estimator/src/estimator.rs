// Common config imports
use crate::config::*;

// Environment-specific config imports (CHAIN_ID only)
#[cfg(feature = "local")]
use crate::config::local::CHAIN_ID;
#[cfg(feature = "paseo")]
use crate::config::paseo::CHAIN_ID;
#[cfg(feature = "polkadot")]
use crate::config::polkadot::CHAIN_ID;
#[cfg(feature = "westend")]
use crate::config::westend::CHAIN_ID;

use crate::contracts::r#i_gateway_v2::IGatewayV2;
use crate::runtimes::*;
use crate::xcm_builder;
// Import types needed for dry-run
use crate::runtimes::{OriginCaller, RawOrigin, RuntimeCall};
use alloy_sol_types::{sol, SolValue};

// Re-export specific enum variants and types for convenience
use crate::runtimes::{
    Junction::GlobalConsensus,
    Junctions::{Here, X1},
    NetworkId,
};
use hex;
use serde::{Deserialize, Serialize};
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

pub async fn clients(
    asset_hub_url: String,
    bridge_hub_url: String,
) -> Result<Clients, EstimatorError> {
    let asset_hub_client: OnlineClient<AssetHubConfig> =
        OnlineClient::from_url(asset_hub_url).await.map_err(|e| {
            EstimatorError::ConnectionError(format!("Cannot connect to asset hub: {}", e))
        })?;

    let bridge_hub_client: OnlineClient<PolkadotConfig> =
        OnlineClient::from_url(bridge_hub_url).await.map_err(|e| {
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
pub struct BridgeHubInfo {
    pub delivery_fee_in_dot: u128,
    pub delivery_fee_in_ether: u128,
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasEstimation {
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    pub bridge_hub: BridgeHubInfo,
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
    let claimer_location = xcm_builder::get_claimer_location(claimer)?;
    let destination_xcm = xcm_builder::build_asset_hub_xcm(
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
        bridge_hub: BridgeHubInfo {
            delivery_fee_in_dot,
            delivery_fee_in_ether,
            dry_run_success: dry_run_result.success,
            dry_run_error: dry_run_result.error_message,
        },
    })
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
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to decode proof: {:?}", e)))?;

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
    use subxt::tx::Payload;

    let submit_call = bridge_hub_runtime::tx()
        .ethereum_inbound_queue_v2()
        .submit(event_proof.clone());

    // Encode the submit call into RuntimeCall
    let call_data = submit_call.encode_call_data(&clients.bridge_hub_client.metadata())
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to encode call data: {:?}", e)))?;

    // Decode into RuntimeCall
    let runtime_call: RuntimeCall = codec::Decode::decode(&mut &call_data[..])
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to decode RuntimeCall: {:?}", e)))?;

    // Use a signed origin (user submitting the extrinsic)
    use subxt_signer::sr25519::dev;
    let ferdie = dev::ferdie();
    let ferdie_account_id: [u8; 32] = ferdie.public_key().0;

    let origin = OriginCaller::system(RawOrigin::Signed(ferdie_account_id.into()));

    let runtime_api_call = bridge_hub_runtime::apis()
        .dry_run_api()
        .dry_run_call(origin, runtime_call, 5u32);

    let dry_run_result = clients
        .bridge_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to dry run call: {:?}", e)))?;

    match dry_run_result {
        Ok(effects) => {
            // Check if the extrinsic dispatch succeeded
            let success = effects.execution_result.is_ok();

            let error_message = if success {
                None
            } else {
                Some(format!(
                    "Dispatch failed: {:?}",
                    effects.execution_result
                ))
            };

            Ok(DryRunResult {
                success,
                error_message,
            })
        }
        Err(e) => Ok(DryRunResult {
            success: false,
            error_message: Some(format!("Dry run API error: {:?}", e)),
        }),
    }
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
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to estimate fee: {:?}", e)))?;

    Ok(fee)
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

fn parse_hex_string(hex_str: &str) -> Result<Vec<u8>, EstimatorError> {
    let hex_str = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };

    hex::decode(hex_str)
        .map_err(|e| EstimatorError::InvalidCommand(format!("Invalid hex string: {}", e)))
}
