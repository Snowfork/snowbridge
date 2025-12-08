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

use crate::estimator::{BridgeAsset, Clients, EstimatorError};
use crate::runtimes::*;
use codec::DecodeLimit;
use sp_core::{parameter_types, H256};
use sp_runtime::AccountId32 as RuntimeAccountId32;

use crate::runtimes::{
    AssetFilter::{Definite, Wild},
    Fungibility::Fungible,
    Hint::AssetClaimer,
    Instruction::{
        DepositAsset, DescendOrigin, ExchangeAsset, PayFees, RefundSurplus, ReserveAssetDeposited,
        SetHints, Transact, UniversalOrigin, WithdrawAsset,
    },
    Junction::{AccountId32, AccountKey20, GlobalConsensus, PalletInstance},
    Junctions::{Here, X1, X2},
    NetworkId::{self, Ethereum},
    WildAsset::AllCounted,
};

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

/// Build the XCM message that will be executed on AssetHub
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

/// Extract the remote XCM from SCALE-encoded bytes
fn extract_remote_xcm(raw: &[u8]) -> Xcm {
    if let Ok(versioned_xcm) = VersionedXcm::decode_with_depth_limit(8, &mut &raw[..]) {
        if let VersionedXcm::V5(xcm) = versioned_xcm {
            return xcm;
        }
    }
    Xcm(vec![])
}

/// Convert a BridgeAsset to an XCM Asset
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

/// Look up the location of a foreign asset by its ID
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

/// Get the claimer location, defaulting to the bridge owner if none provided
pub fn get_claimer_location(claimer: Option<Location>) -> Result<Location, EstimatorError> {
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
