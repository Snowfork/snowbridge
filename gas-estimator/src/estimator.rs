use crate::{Commands, Environment};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::Fungibility::Fungible;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::asset::{
    Asset, AssetId, Assets, Fungibility,
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::GlobalConsensus;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::PalletInstance;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junction::NetworkId::Ethereum;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::location::Location;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::UniversalOrigin;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Instruction::{
    DescendOrigin, RefundSurplus, ReserveAssetDeposited, SetHints
};
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Xcm;
use asset_hub_westend_runtime::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use asset_hub_westend_runtime::runtime_types::staging_xcm::v5::Hint::AssetClaimer;
use subxt::{Config, OnlineClient, PolkadotConfig, config::DefaultExtrinsicParams};
use lazy_static::lazy_static;
use asset_hub_westend_runtime::runtime_types::xcm::VersionedXcm;
use std::env;
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

pub fn build_asset_hub_xcm(xcm_bytes: &[u8], claimer: Location) -> VersionedXcm {
    let mut instructions = vec![
        DescendOrigin(X1([PalletInstance(91)])),
        UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 1 })),
        ReserveAssetDeposited(Assets(
            vec![Asset {
                id: AssetId(Location {
                    parents: 2,
                    interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: 1 })]),
                }),
                fun: Fungible(1500000000000u128),
            }]
            .into(),
        )),
        SetHints { hints: BoundedVec([AssetClaimer { location: claimer }].into()) },
    ];


    instructions.extend(extract_remote_xcm(xcm_bytes));

    VersionedXcm::V5( Xcm::from(Xcm(instructions)))
}

fn extract_remote_xcm(raw: &[u8]) -> Xcm<> {
    if let Ok(versioned_xcm) =
        VersionedXcm::decode_with_depth_limit(8, &mut raw)
    {
        if let Ok(decoded_xcm) = versioned_xcm.try_into() {
            return decoded_xcm;
        }
    }
    Xcm::new()
}
