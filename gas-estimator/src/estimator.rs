use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Fungibility::Fungible;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::{
    Asset, AssetId, Assets,
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::GlobalConsensus;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::PalletInstance;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId::Ethereum;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::UniversalOrigin;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::{
    DescendOrigin, ReserveAssetDeposited, SetHints
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Xcm;
use asset_hub_westend_runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Hint::AssetClaimer;
use subxt::{Config, OnlineClient, PolkadotConfig, config::DefaultExtrinsicParams};
use lazy_static::lazy_static;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedXcm;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedAssetId;
use asset_hub_westend_runtime::runtime_types::sp_weights::weight_v2::Weight;
use std::env;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::Here;
use codec::DecodeLimit;

lazy_static! {
    pub static ref ASSET_HUB_WS_URL: String = {
        if let Ok(val) = env::var("ASSET_HUB_WS_URL") {
            val
        } else {
            "ws://127.0.0.1:12144".to_string()
        }
    };
}

const CHAIN_ID: u64 = 11155111;

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
}

impl std::fmt::Display for EstimatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EstimatorError::InvalidHexFormat => write!(f, "Command must start with 0x"),
            EstimatorError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            EstimatorError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl std::error::Error for EstimatorError {}

pub async fn clients() -> Result<Clients, EstimatorError> {
    let asset_hub_client: OnlineClient<AssetHubConfig> =
        OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string())
            .await
            .map_err(|e| EstimatorError::ConnectionError(format!("Cannot connect to asset hub: {}", e)))?;
    Ok(Clients {
        asset_hub_client: Box::new(asset_hub_client),
    })
}

pub struct Clients {
    pub asset_hub_client: Box<OnlineClient<AssetHubConfig>>,
}

pub async fn estimate_gas(
    clients: &Clients,
    xcm_bytes: &[u8],
    claimer: Location,
) -> Result<String, EstimatorError> {
    let destination_xcm = build_asset_hub_xcm(xcm_bytes, claimer);

    let weight = query_xcm_weight(clients, destination_xcm).await?;
    let fee_in_dot = query_weight_to_asset_fee(clients, &weight).await?;

    let dot_asset = Location {
        parents: 1,
        interior: Here,
    };

    let ether_asset = Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
    };

    // Quote the price to swap DOT for Ether
    let fee_in_ether = quote_price_exact_tokens_for_tokens(
        clients,
        dot_asset,
        ether_asset,
        fee_in_dot,
        true,
    ).await?;

    Ok(format!(
        "XCM weight: {:?}\nFee in DOT: {} planck\nFee in Ether: {} wei",
        weight, fee_in_dot, fee_in_ether
    ))
}

pub fn build_asset_hub_xcm(xcm_bytes: &[u8], claimer: Location) -> VersionedXcm {
    let mut instructions = vec![
        DescendOrigin(X1([PalletInstance(91)])),
        UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 1 })),
        ReserveAssetDeposited(Assets(
            vec![Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: CHAIN_ID })]),
                }),
                fun: Fungible(1500000000000u128),
            }]
            .into(),
        )),
        SetHints { hints: BoundedVec([AssetClaimer { location: claimer }].into()) },
    ];

    let remote_xcm = extract_remote_xcm(xcm_bytes);
    instructions.extend(remote_xcm.0);

    VersionedXcm::V5(Xcm(instructions))
}

fn extract_remote_xcm(raw: &[u8]) -> Xcm {
    if let Ok(versioned_xcm) =
        VersionedXcm::decode_with_depth_limit(8, &mut &raw[..])
    {
        if let VersionedXcm::V5(xcm) = versioned_xcm {
            return xcm;
        }
    }
    Xcm(vec![])
}

pub async fn query_xcm_weight(clients: &Clients, destination_xcm: VersionedXcm) -> Result<Weight, EstimatorError> {
    let runtime_api_call = asset_hub_westend_runtime::runtime::apis().xcm_payment_api().query_xcm_weight(destination_xcm);

    let weight_result = clients.asset_hub_client
        .runtime_api()
        .at_latest()
        .await.map_err(|e| EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e)))?
        .call(runtime_api_call)
        .await
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to query XCM weight: {:?}", e)))?;

    weight_result.map_err(|e| EstimatorError::InvalidCommand(format!("XCM weight query returned error: {:?}", e)))
}

pub async fn query_weight_to_asset_fee(clients: &Clients, weight: &Weight) -> Result<u128, EstimatorError> {
    let dot_asset = VersionedAssetId::V5(AssetId(Location {
        parents: 1,
        interior: Here,
    }));

    let runtime_api_call = asset_hub_westend_runtime::runtime::apis().xcm_payment_api().query_weight_to_asset_fee(weight.clone(), dot_asset);

    let fee_result = clients.asset_hub_client
        .runtime_api()
        .at_latest()
        .await.map_err(|e| EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e)))?
        .call(runtime_api_call)
        .await
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to query weight to asset fee: {:?}", e)))?;

    fee_result.map_err(|e| EstimatorError::InvalidCommand(format!("Weight to asset fee query returned error: {:?}", e)))
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

    let quote_result = clients.asset_hub_client
        .runtime_api()
        .at_latest()
        .await.map_err(|e| EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e)))?
        .call(runtime_api_call)
        .await
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to quote price for tokens: {:?}", e)))?;

    quote_result.ok_or_else(|| EstimatorError::InvalidCommand("Quote price query returned None".to_string()))
}

