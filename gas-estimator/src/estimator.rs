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
use codec::Decode;
use hex;
use serde::{Deserialize, Serialize};
use subxt::tx::Payload;
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
    let asset_hub_client =
        connect_with_failover::<AssetHubConfig>(&asset_hub_url, "asset hub").await?;
    let bridge_hub_client =
        connect_with_failover::<PolkadotConfig>(&bridge_hub_url, "bridge hub").await?;

    Ok(Clients {
        asset_hub_client: Box::new(asset_hub_client),
        bridge_hub_client: Box::new(bridge_hub_client),
    })
}

// connect_with_failover accepts a comma-separated list of endpoint URLs, tries
// them in random order, and returns the first client that connects. Mirrors
// the relayer's endpoint failover pattern so a single flaky endpoint doesn't
// take down gas estimation (which would dead-letter the nonce).
async fn connect_with_failover<C: subxt::Config>(
    urls: &str,
    label: &str,
) -> Result<OnlineClient<C>, EstimatorError> {
    use rand::seq::SliceRandom;

    let mut endpoints: Vec<&str> = urls
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if endpoints.is_empty() {
        return Err(EstimatorError::ConnectionError(format!(
            "no {} endpoints configured",
            label
        )));
    }
    endpoints.shuffle(&mut rand::thread_rng());

    let mut last_err: Option<String> = None;
    for endpoint in &endpoints {
        match OnlineClient::<C>::from_url(*endpoint).await {
            Ok(client) => return Ok(client),
            Err(e) => {
                log::warn!("{} endpoint {} failed: {}", label, endpoint, e);
                last_err = Some(format!("{}: {}", endpoint, e));
            }
        }
    }
    Err(EstimatorError::ConnectionError(format!(
        "Cannot connect to {} (tried {} endpoint(s)): {}",
        label,
        endpoints.len(),
        last_err.unwrap_or_else(|| "unknown".to_string())
    )))
}

pub struct Clients {
    pub asset_hub_client: Box<OnlineClient<AssetHubConfig>>,
    pub bridge_hub_client: Box<OnlineClient<PolkadotConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeHubInfo {
    pub delivery_fee_in_dot: u128,
    pub delivery_fee_in_ether: u128,
    pub tip_in_ether: Option<u128>,
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasEstimation {
    /// Always 0 from `estimate message`. ethereum-v3 takes Bridge Hub extrinsic
    /// cost from `estimate batch` of the real force_batch (beacon ± submits).
    /// Kept for JSON compatibility with older callers.
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    pub bridge_hub: BridgeHubInfo,
    pub beneficiaries: Vec<String>,
    /// Ether the message deposits to a configured service fee recipient on Asset
    /// Hub. This is Snowbridge revenue that the relayer does not receive.
    pub service_fee_in_ether: u128,
    /// Asset Hub `dry_run_xcm` of the destination XCM. This leg does not depend on
    /// the Bridge Hub beacon header, so a bare `ethereumInboundQueueV2.submit`
    /// that fails only because the header is not on chain yet still reports here.
    pub asset_hub_dry_run_success: bool,
    pub asset_hub_dry_run_error: Option<String>,
}

#[derive(Debug, Clone)]
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
    event_log_tx_index: u64,
    proof_hex: &str,
    xcm_bytes: &[u8],
    claimer: Option<Location>,
    origin: [u8; 20],
    value: u128,
    execution_fee: u128,
    _relayer_fee: u128,
    assets: &[BridgeAsset],
    _relayer_account: [u8; 32],
    nonce: u64,
    service_fee_recipients: &[[u8; 32]],
) -> Result<GasEstimation, EstimatorError> {
    // Event-proof / bare-submit payment-info is intentionally not quoted here.
    // A solo `EthereumInboundQueueV2.submit` fee is wrong when the real extrinsic
    // is a force_batch that may prepend EthereumBeaconClient.submit, and
    // ethereum-v3's gate uses `estimate batch` for Bridge Hub cost instead.
    // CLI still accepts the proof args for compatibility.
    let _ = (
        event_log_address,
        event_log_topics,
        event_log_data,
        event_log_tx_index,
        proof_hex,
        _relayer_account,
        _relayer_fee,
    );

    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };

    let ether_asset = Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
    };

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

    // Asset Hub destination XCM only. Bridge Hub dispatch + extrinsic fee are
    // checked by `estimate batch` of the force_batch ethereum-v3 actually submits
    // (beacon update optional when the on-chain LC already covers the message).
    // See: https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/xcm-runtime-apis/src/dry_run.rs
    let asset_hub_dry_run = dry_run_destination_xcm_on_asset_hub(clients, &destination_xcm).await?;
    let asset_hub_dry_run_success = asset_hub_dry_run.success;
    let asset_hub_dry_run_error = asset_hub_dry_run.error_message.clone();
    // bridge_hub.dry_run_* mirrors the Asset Hub result for callers that still
    // read the combined field (legacy IsProfitable).
    let dry_run_result = asset_hub_dry_run;

    // Query for tips using the provided nonce
    let tip_in_ether = query_tip(clients, nonce).await?;

    // Extract beneficiary addresses for OFAC checks
    let beneficiaries = extract_beneficiaries(&destination_xcm);

    let service_fee_in_ether = extract_service_fee(&destination_xcm, service_fee_recipients);

    Ok(GasEstimation {
        extrinsic_fee_in_dot: 0,
        extrinsic_fee_in_ether: 0,
        bridge_hub: BridgeHubInfo {
            delivery_fee_in_dot,
            delivery_fee_in_ether,
            tip_in_ether,
            dry_run_success: dry_run_result.success,
            dry_run_error: dry_run_result.error_message,
        },
        beneficiaries,
        service_fee_in_ether,
        asset_hub_dry_run_success,
        asset_hub_dry_run_error,
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

    // Query delivery fees using XCM Payment API, latest API specifies the fee asset
    #[cfg(any(feature = "local", feature = "westend"))]
    let runtime_api_call = {
        let dot_asset = BridgeHubLocation {
            parents: 1,
            interior: BridgeHubJunctions::Here,
        };
        bridge_hub_runtime::apis()
            .xcm_payment_api()
            .query_delivery_fees(
                versioned_destination,
                bridge_hub_xcm,
                VersionedAssetId::V5(BridgeHubAssetId(dot_asset)),
            )
    };

    #[cfg(any(feature = "paseo", feature = "polkadot"))]
    let runtime_api_call = {
        let dot_asset = BridgeHubLocation {
            parents: 1,
            interior: BridgeHubJunctions::Here,
        };
        bridge_hub_runtime::apis()
            .xcm_payment_api()
            .query_delivery_fees(
                versioned_destination,
                bridge_hub_xcm,
                VersionedAssetId::V5(BridgeHubAssetId(dot_asset)),
            )
    };

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

#[allow(dead_code)] // used by calculate_extrinsic_fee_in_dot / examples
fn construct_event_proof(
    event_log_address: &str,
    event_log_topics: &str,
    event_log_data: &str,
    event_log_tx_index: u64,
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
        tx_index: event_log_tx_index as _,
        data: data.into(),
    };

    Ok(EventProof {
        event_log: log,
        proof,
    })
}

/// Dry-run the constructed AssetHub destination XCM.
///
/// Origin is BridgeHub as seen from AssetHub (`parents: 1, Parachain(1002)`), matching the
/// HRMP/XCMP sender of inbound-queue messages. Success requires `Outcome::Complete`.
async fn dry_run_destination_xcm_on_asset_hub(
    clients: &Clients,
    destination_xcm: &VersionedXcm,
) -> Result<DryRunResult, EstimatorError> {
    let origin_location = VersionedLocation::V5(Location {
        parents: 1,
        interior: X1([Junction::Parachain(BRIDGE_HUB_PARA_ID)]),
    });

    let runtime_api_call = asset_hub_runtime::apis()
        .dry_run_api()
        .dry_run_xcm(origin_location, destination_xcm.clone());

    let dry_run_result = clients
        .asset_hub_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest AssetHub block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to dry run AssetHub XCM: {:?}", e))
        })?;

    match dry_run_result {
        Ok(effects) => {
            let success = matches!(effects.execution_result, Outcome::Complete { .. });
            let error_message = if success {
                None
            } else {
                Some(format!(
                    "AssetHub dry_run_xcm failed: {:?}",
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
            error_message: Some(format!("AssetHub dry_run_xcm API error: {:?}", e)),
        }),
    }
}

/// Payment-info quote for a bare `EthereumInboundQueueV2.submit`.
/// Unused by `estimate_gas` (ethereum-v3 prices BH cost via `dry_run_force_batch`);
/// kept for ad-hoc / example tooling.
#[allow(dead_code)]
async fn calculate_extrinsic_fee_in_dot(
    clients: &Clients,
    event_proof: &EventProof,
) -> Result<u128, EstimatorError> {
    let submit_call = bridge_hub_runtime::tx()
        .ethereum_inbound_queue_v2()
        .submit(event_proof.clone());

    estimate_payload_fee_in_dot(clients, &submit_call).await
}

/// One inner call of a dry-run `Utility.force_batch`, in submission order.
/// `ItemCompleted` / `ItemFailed` are emitted once per call by the utility pallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemDryRun {
    pub index: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// Dry-run of the extrinsic ethereum-v3 actually submits: `Utility.force_batch`
/// of the beacon update and V1/V2 submits. The single-message command still
/// dry-runs a bare `EthereumInboundQueueV2.submit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDryRun {
    pub extrinsic_fee_in_dot: u128,
    pub extrinsic_fee_in_ether: u128,
    /// Outer dispatch succeeded and every inner item completed.
    pub dry_run_success: bool,
    pub dry_run_error: Option<String>,
    pub items: Vec<BatchItemDryRun>,
}

pub fn decode_runtime_call(bytes: &[u8]) -> Result<RuntimeCall, EstimatorError> {
    if bytes.is_empty() {
        return Err(EstimatorError::InvalidCommand(
            "RuntimeCall bytes are empty".to_string(),
        ));
    }
    RuntimeCall::decode(&mut &bytes[..]).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to decode RuntimeCall: {:?}", e))
    })
}

/// Classify utility-pallet events from a force_batch dry-run.
///
/// `event_debug` is the Debug text of each emitted runtime event. Inner pallet
/// events are ignored. `ItemCompleted` / `ItemFailed` stay in call order;
/// `BatchCompletedWithErrors` is checked before `BatchCompleted` because the
/// latter is a substring of the former.
pub fn classify_force_batch_events(event_debug: &[String]) -> (Vec<BatchItemDryRun>, bool) {
    let mut items = Vec::new();
    let mut with_errors = false;
    for text in event_debug {
        let utility = text.contains("Utility") || text.contains("pallet_utility");
        if !utility {
            continue;
        }
        if text.contains("ItemFailed") {
            items.push(BatchItemDryRun {
                index: items.len() as u32,
                success: false,
                error: Some(text.clone()),
            });
        } else if text.contains("ItemCompleted") {
            items.push(BatchItemDryRun {
                index: items.len() as u32,
                success: true,
                error: None,
            });
        } else if text.contains("BatchCompletedWithErrors") {
            with_errors = true;
        }
    }
    (items, with_errors)
}

/// Dry-run `Utility.force_batch(calls)` as a signed relayer and price that extrinsic.
///
/// `calls` must be the SCALE `RuntimeCall`s the relayer will batch, in order
/// (beacon update, then V2 submits, then V1 submits). Unlike `estimate message`,
/// a V2 submit is dry-run after the earlier calls in this batch, so a message
/// that needs the beacon update in front of it is checked against that update.
pub async fn dry_run_force_batch(
    clients: &Clients,
    calls: Vec<RuntimeCall>,
    relayer_account: [u8; 32],
) -> Result<BatchDryRun, EstimatorError> {
    if calls.is_empty() {
        return Err(EstimatorError::InvalidCommand(
            "force_batch requires at least one call".to_string(),
        ));
    }
    let call_count = calls.len();

    let batch_call = bridge_hub_runtime::tx().utility().force_batch(calls);

    let extrinsic_fee_in_dot = estimate_payload_fee_in_dot(clients, &batch_call).await?;

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
        dot_asset,
        ether_asset,
        extrinsic_fee_in_dot,
        true,
    )
    .await?;

    let call_data = batch_call
        .encode_call_data(&clients.bridge_hub_client.metadata())
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to encode force_batch: {:?}", e))
        })?;
    let runtime_call: RuntimeCall = codec::Decode::decode(&mut &call_data[..]).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to decode force_batch RuntimeCall: {:?}", e))
    })?;

    let origin = OriginCaller::system(RawOrigin::Signed(relayer_account.into()));
    let runtime_api_call =
        bridge_hub_runtime::apis()
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
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to dry run force_batch: {:?}", e))
        })?;

    let (outer_ok, outer_error, event_debug) = match dry_run_result {
        Ok(effects) => {
            let outer_ok = effects.execution_result.is_ok();
            let outer_error = if outer_ok {
                None
            } else {
                Some(format!(
                    "force_batch dispatch failed: {:?}",
                    effects.execution_result
                ))
            };
            let event_debug = effects
                .emitted_events
                .iter()
                .map(|event| format!("{:?}", event))
                .collect::<Vec<_>>();
            (outer_ok, outer_error, event_debug)
        }
        Err(e) => {
            return Ok(BatchDryRun {
                extrinsic_fee_in_dot,
                extrinsic_fee_in_ether,
                dry_run_success: false,
                dry_run_error: Some(format!("BridgeHub dry_run_call API error: {:?}", e)),
                items: Vec::new(),
            });
        }
    };

    let (items, with_errors) = classify_force_batch_events(&event_debug);
    let mut dry_run_error = outer_error;
    let items_match = items.len() == call_count;
    if outer_ok && !items_match {
        dry_run_error = Some(format!(
            "force_batch emitted {} item result(s) for {} call(s)",
            items.len(),
            call_count
        ));
    } else if outer_ok && with_errors {
        let failed: Vec<u32> = items
            .iter()
            .filter(|item| !item.success)
            .map(|item| item.index)
            .collect();
        dry_run_error = Some(format!(
            "force_batch completed with failed items: {:?}",
            failed
        ));
    }

    let dry_run_success =
        outer_ok && items_match && !with_errors && items.iter().all(|item| item.success);

    Ok(BatchDryRun {
        extrinsic_fee_in_dot,
        extrinsic_fee_in_ether,
        dry_run_success,
        dry_run_error,
        items,
    })
}

async fn estimate_payload_fee_in_dot<Call>(
    clients: &Clients,
    call: &Call,
) -> Result<u128, EstimatorError>
where
    Call: Payload,
{
    let alice = dev::alice();
    let fee = clients
        .bridge_hub_client
        .tx()
        .create_signed(call, &alice, Default::default())
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

/// Extract beneficiary addresses from XCM instructions
/// Returns addresses in hex format for OFAC checks
/// `Location` has no `PartialEq`, so bridged ether is matched structurally.
fn is_bridged_ether(location: &Location) -> bool {
    if location.parents != 2 {
        return false;
    }
    match &location.interior {
        X1(junctions) => junctions.iter().any(|junction| {
            matches!(
                junction,
                GlobalConsensus(NetworkId::Ethereum { chain_id }) if *chain_id == CHAIN_ID
            )
        }),
        _ => false,
    }
}

/// Sums the ether a message deposits to a service fee recipient. The SDK emits it
/// as a top-level `DepositAsset` with a `Definite` filter, before the beneficiary
/// deposit. Only top-level instructions are walked, so the `SetAppendix` refund
/// deposit to the same account is not counted.
pub fn extract_service_fee(xcm: &VersionedXcm, recipients: &[[u8; 32]]) -> u128 {
    if recipients.is_empty() {
        return 0;
    }

    let instructions = match xcm {
        VersionedXcm::V5(xcm) => &xcm.0,
        _ => return 0,
    };

    let mut total: u128 = 0;
    for instruction in instructions {
        let Instruction::DepositAsset {
            assets: AssetFilter::Definite(assets),
            beneficiary,
        } = instruction
        else {
            continue;
        };
        let Junctions::X1(junctions) = &beneficiary.interior else {
            continue;
        };
        let is_recipient = junctions.iter().any(|junction| {
            matches!(junction, Junction::AccountId32 { id, .. } if recipients.contains(id))
        });
        if !is_recipient {
            continue;
        }
        for asset in assets.0.iter() {
            if !is_bridged_ether(&asset.id.0) {
                continue;
            }
            if let Fungibility::Fungible(amount) = asset.fun {
                total = total.saturating_add(amount);
            }
        }
    }
    total
}

pub fn extract_beneficiaries(xcm: &VersionedXcm) -> Vec<String> {
    let mut beneficiaries = Vec::new();

    let instructions = match xcm {
        VersionedXcm::V5(xcm) => &xcm.0,
        _ => return beneficiaries,
    };

    for instruction in instructions {
        if let Instruction::DepositAsset { beneficiary, .. } = instruction {
            // Extract address from beneficiary location
            match &beneficiary.interior {
                Junctions::X1(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X2(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X3(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X4(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X5(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X6(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X7(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                Junctions::X8(junctions) => {
                    for junction in junctions.iter() {
                        if let Junction::AccountId32 { id, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(id)));
                        } else if let Junction::AccountKey20 { key, .. } = junction {
                            beneficiaries.push(format!("0x{}", hex::encode(key)));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Remove duplicates
    beneficiaries.sort();
    beneficiaries.dedup();

    beneficiaries
}

async fn query_tip(clients: &Clients, nonce: u64) -> Result<Option<u128>, EstimatorError> {
    // Query the Tips storage map
    let storage_query = bridge_hub_runtime::storage()
        .ethereum_inbound_queue_v2()
        .tips(nonce);

    let tip_in_ether = clients
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
            EstimatorError::InvalidCommand(format!("Failed to query tips storage: {:?}", e))
        })?;

    Ok(tip_in_ether)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_force_batch_events_keeps_item_order_and_with_errors() {
        let events = vec![
            "RuntimeEvent::EthereumInboundQueueV2(MessageReceived)".to_string(),
            "RuntimeEvent::Utility(ItemCompleted)".to_string(),
            "RuntimeEvent::Utility(ItemFailed { error: Module })".to_string(),
            "RuntimeEvent::Utility(BatchCompletedWithErrors)".to_string(),
        ];
        let (items, with_errors) = classify_force_batch_events(&events);
        assert!(with_errors);
        assert_eq!(items.len(), 2);
        assert!(items[0].success);
        assert_eq!(items[0].index, 0);
        assert!(!items[1].success);
        assert_eq!(items[1].index, 1);
        assert!(items[1].error.as_ref().unwrap().contains("ItemFailed"));
    }

    #[test]
    fn classify_force_batch_events_ignores_batch_completed_substring() {
        let events = vec!["RuntimeEvent::Utility(BatchCompleted)".to_string()];
        let (items, with_errors) = classify_force_batch_events(&events);
        assert!(items.is_empty());
        assert!(!with_errors);
    }

    fn ether() -> Location {
        Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
        }
    }

    fn deposit(amount: u128, account: [u8; 32], asset: Location) -> Instruction {
        Instruction::DepositAsset {
            assets: AssetFilter::Definite(Assets(vec![Asset {
                id: AssetId(asset),
                fun: Fungibility::Fungible(amount),
            }])),
            beneficiary: Location {
                parents: 0,
                interior: X1([Junction::AccountId32 {
                    network: None,
                    id: account,
                }]),
            },
        }
    }

    #[test]
    fn extract_service_fee_sums_deposits_to_configured_recipients() {
        let recipient = [1u8; 32];
        let other = [2u8; 32];

        let xcm = VersionedXcm::V5(Xcm(vec![
            deposit(15_000_000_000_000, recipient, ether()),
            deposit(999, other, ether()),
        ]));

        assert_eq!(extract_service_fee(&xcm, &[recipient]), 15_000_000_000_000);
        assert_eq!(extract_service_fee(&xcm, &[other]), 999);
        assert_eq!(extract_service_fee(&xcm, &[]), 0);
        assert_eq!(extract_service_fee(&xcm, &[[3u8; 32]]), 0);
    }

    #[test]
    fn extract_service_fee_ignores_non_ether_and_wild_deposits() {
        let recipient = [1u8; 32];
        let dot = Location {
            parents: 1,
            interior: Here,
        };

        let xcm = VersionedXcm::V5(Xcm(vec![
            deposit(500, recipient, dot),
            Instruction::DepositAsset {
                assets: AssetFilter::Wild(WildAsset::AllCounted(2)),
                beneficiary: Location {
                    parents: 0,
                    interior: X1([Junction::AccountId32 {
                        network: None,
                        id: recipient,
                    }]),
                },
            },
        ]));

        assert_eq!(extract_service_fee(&xcm, &[recipient]), 0);
    }

    /// Live check: two System.remark calls inside Utility.force_batch both complete.
    /// `cargo test --features polkadot --lib live_force_batch_remark -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "talks to live Bridge Hub and Asset Hub"]
    async fn live_force_batch_remark() {
        let clients = clients(
            "wss://polkadot-asset-hub-rpc.polkadot.io".to_string(),
            "wss://rpc-bridge-hub-polkadot.stakeworld.io".to_string(),
        )
        .await
        .expect("connect");
        let remark = bridge_hub_runtime::tx()
            .system()
            .remark(b"snowbridge-gas-estimator-batch".to_vec());
        let data = remark
            .encode_call_data(&clients.bridge_hub_client.metadata())
            .expect("encode remark");
        let call = RuntimeCall::decode(&mut &data[..]).expect("decode remark");
        let relayer = dev::alice().public_key().0;
        let result = dry_run_force_batch(&clients, vec![call.clone(), call], relayer)
            .await
            .expect("dry-run");
        assert!(
            result.dry_run_success,
            "fee {} items {:?} err {:?}",
            result.extrinsic_fee_in_dot, result.items, result.dry_run_error
        );
        assert_eq!(result.items.len(), 2);
        assert!(result.extrinsic_fee_in_dot > 0);
    }
}
