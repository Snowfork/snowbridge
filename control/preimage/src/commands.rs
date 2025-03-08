use crate::helpers::calculate_delivery_fee;
use crate::{
    constants::*, Context, ForceCheckpointArgs, GatewayAddressArgs, GatewayOperatingModeEnum,
    OperatingModeEnum, PricingParametersArgs, RegisterEtherArgs, UpdateAssetArgs, UpgradeArgs,
};
use alloy_primitives::{utils::format_units, U256};
use codec::Encode;
use snowbridge_router_primitives::inbound::GlobalConsensusEthereumConvertsFor;
use sp_arithmetic::FixedU128;
use sp_crypto_hashing::twox_128;
use std::{fs::File, io::Read};
use subxt::utils::MultiAddress;
use subxt::utils::Static;

type CheckpointUpdate = snowbridge_beacon_primitives::CheckpointUpdate<512>;

use crate::asset_hub_runtime::runtime_types::pallet_assets;
use crate::asset_hub_runtime::RuntimeCall as AssetHubRuntimeCall;

use crate::bridge_hub_runtime::runtime_types::{
    snowbridge_core::{
        operating_mode::BasicOperatingMode,
        outbound::v1::{Initializer, OperatingMode},
        pricing::{PricingParameters, Rewards},
    },
    snowbridge_pallet_ethereum_client, snowbridge_pallet_inbound_queue,
    snowbridge_pallet_outbound_queue, snowbridge_pallet_system,
};
use crate::bridge_hub_runtime::RuntimeCall as BridgeHubRuntimeCall;

#[cfg(feature = "polkadot")]
pub mod asset_hub_polkadot_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v4::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
        location::Location,
    };
    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
            parents: 2,
            interior: X2([
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ]),
        };
    }
}

#[cfg(feature = "paseo")]
pub mod asset_hub_paseo_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v3::multilocation::MultiLocation;
    pub use crate::asset_hub_runtime::runtime_types::xcm::v3::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
    };
    pub fn get_ether_id(chain_id: u64) -> MultiLocation {
        return MultiLocation {
            parents: 2,
            interior: X1(GlobalConsensus(NetworkId::Ethereum { chain_id })),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> MultiLocation {
        return MultiLocation {
            parents: 2,
            interior: X2(
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ),
        };
    }
}

#[cfg(feature = "westend")]
pub mod asset_hub_westend_types {
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
        location::Location,
    };
    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
            parents: 2,
            interior: X2([
                GlobalConsensus(NetworkId::Ethereum { chain_id }),
                AccountKey20 { network: None, key },
            ]),
        };
    }
}

pub fn gateway_operating_mode(operating_mode: &GatewayOperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match operating_mode {
        GatewayOperatingModeEnum::Normal => OperatingMode::Normal,
        GatewayOperatingModeEnum::RejectingOutboundMessages => {
            OperatingMode::RejectingOutboundMessages
        }
    };
    BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::set_operating_mode { mode },
    )
}

pub fn inbound_queue_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumInboundQueue(
        snowbridge_pallet_inbound_queue::pallet::Call::set_operating_mode { mode },
    )
}

pub fn ethereum_client_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::set_operating_mode { mode },
    )
}

pub fn outbound_queue_operating_mode(param: &OperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match param {
        OperatingModeEnum::Normal => BasicOperatingMode::Normal,
        OperatingModeEnum::Halted => BasicOperatingMode::Halted,
    };
    BridgeHubRuntimeCall::EthereumOutboundQueue(
        snowbridge_pallet_outbound_queue::pallet::Call::set_operating_mode { mode },
    )
}

pub fn upgrade(params: &UpgradeArgs) -> BridgeHubRuntimeCall {
    BridgeHubRuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::upgrade {
        impl_address: params.logic_address.into_array().into(),
        impl_code_hash: params.logic_code_hash.0.into(),
        initializer: Some(Initializer {
            params: params.initializer_params.clone().into(),
            maximum_required_gas: params.initializer_gas,
        }),
    })
}

pub async fn pricing_parameters(
    context: &Context,
    params: &PricingParametersArgs,
) -> Result<(BridgeHubRuntimeCall, AssetHubRuntimeCall), Box<dyn std::error::Error>> {
    // BridgeHub parameters
    let pricing_params: PricingParameters<u128> = PricingParameters {
        exchange_rate: Static(FixedU128::from_rational(
            params.exchange_rate_numerator.into(),
            params.exchange_rate_denominator.into(),
        )),
        multiplier: Static(FixedU128::from_rational(
            params.multiplier_numerator.into(),
            params.multiplier_denominator.into(),
        )),
        fee_per_gas: crate::bridge_hub_runtime::runtime_types::primitive_types::U256(
            params.fee_per_gas.into_limbs(),
        ),
        rewards: Rewards {
            local: params.local_reward.to::<u128>(),
            remote: crate::bridge_hub_runtime::runtime_types::primitive_types::U256(
                params.remote_reward.into_limbs(),
            ),
        },
    };

    let outbound_delivery_fee =
        calculate_delivery_fee(&context.bridge_hub_api, &pricing_params).await?;

    let total_outbound_fee = outbound_delivery_fee.local + outbound_delivery_fee.remote;

    // Adjust outbound fee up by 10% as a buffer
    let total_outbound_fee_adjusted = total_outbound_fee.saturating_add(total_outbound_fee / 10);

    eprintln!("BridgeHub:");
    eprintln!(
        "  ExchangeRate: {} ETH/{}",
        params.exchange_rate_numerator as f64 / params.exchange_rate_denominator as f64,
        POLKADOT_SYMBOL
    );
    eprintln!(
        "  FeePerGas: {} GWEI",
        format_units(params.fee_per_gas, "gwei").unwrap(),
    );
    eprintln!(
        "  LocalReward: {} {} [{} PLANCK]",
        format_units(U256::from(params.local_reward), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        params.local_reward,
    );
    eprintln!(
        "  RemoteReward: {} ETH [{} WEI]",
        format_units(params.remote_reward, "eth").unwrap(),
        params.remote_reward
    );
    eprintln!("AssetHub:");
    eprintln!(
        "  BaseFee: {} {}, [{} PLANCK]",
        format_units(U256::from(total_outbound_fee_adjusted), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        total_outbound_fee_adjusted
    );

    // AssetHub parameters
    let asset_hub_outbound_fee_storage_key: Vec<u8> =
        twox_128(b":BridgeHubEthereumBaseFee:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = total_outbound_fee_adjusted.encode();

    eprintln!(
        "Storage key for 'BridgeHubEthereumBaseFee': 0x{}",
        hex::encode(&asset_hub_outbound_fee_storage_key)
    );

    Ok((
        BridgeHubRuntimeCall::EthereumSystem(
            snowbridge_pallet_system::pallet::Call::set_pricing_parameters {
                params: pricing_params,
            },
        ),
        AssetHubRuntimeCall::System(
            crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
                items: vec![(
                    asset_hub_outbound_fee_storage_key,
                    asset_hub_outbound_fee_encoded,
                )],
            },
        ),
    ))
}

pub fn set_assethub_fee(fee: u128) -> AssetHubRuntimeCall {
    let asset_hub_outbound_fee_storage_key: Vec<u8> =
        twox_128(b":BridgeHubEthereumBaseFee:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = fee.encode();

    AssetHubRuntimeCall::System(
        crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(
                asset_hub_outbound_fee_storage_key,
                asset_hub_outbound_fee_encoded,
            )],
        },
    )
}

pub fn force_checkpoint(params: &ForceCheckpointArgs) -> BridgeHubRuntimeCall {
    let mut file = File::open(params.checkpoint.clone()).expect("File not found");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Failed to read the file");
    let checkpoint: CheckpointUpdate = serde_json::from_str(&data).unwrap();
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::force_checkpoint {
            update: Box::new(Static(checkpoint)),
        },
    )
}

pub fn set_gateway_address(params: &GatewayAddressArgs) -> BridgeHubRuntimeCall {
    let storage_key = sp_crypto_hashing::twox_128(b":EthereumGatewayAddress:").to_vec();
    let storage_value = params.gateway_address.into_array().encode();
    BridgeHubRuntimeCall::System(
        crate::bridge_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(storage_key, storage_value)],
        },
    )
}

pub fn make_asset_sufficient(params: &UpdateAssetArgs) -> AssetHubRuntimeCall {
    use subxt::utils::AccountId32;
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;
    let asset_id = get_asset_id(chain_id, params.contract_id.into_array().into());
    let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);
    AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_asset_status {
        id: asset_id,
        owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        issuer: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        admin: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        freezer: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        min_balance: params.min_balance,
        is_sufficient: params.is_sufficient,
        is_frozen: params.is_frozen,
    })
}

pub fn force_set_metadata(params: &UpdateAssetArgs) -> AssetHubRuntimeCall {
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;
    let asset_id = get_asset_id(chain_id, params.contract_id.into_array().into());
    AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_set_metadata {
        id: asset_id,
        name: params.name.as_bytes().to_vec(),
        symbol: params.symbol.as_bytes().to_vec(),
        decimals: params.decimals,
        is_frozen: params.is_frozen,
    })
}

pub fn register_ether(params: &RegisterEtherArgs) -> (AssetHubRuntimeCall, AssetHubRuntimeCall) {
    use subxt::utils::AccountId32;
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    #[cfg(feature = "paseo")]
    use asset_hub_paseo_types::*;
    #[cfg(feature = "polkadot")]
    use asset_hub_polkadot_types::*;
    #[cfg(feature = "westend")]
    use asset_hub_westend_types::*;

    let asset_id = get_ether_id(chain_id);
    let owner = GlobalConsensusEthereumConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);

    let force_register =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_create {
            id: asset_id.clone(),
            min_balance: params.ether_min_balance,
            is_sufficient: true,
            owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        });
    let metadata =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_set_metadata {
            id: asset_id,
            name: params.ether_name.as_bytes().to_vec(),
            symbol: params.ether_symbol.as_bytes().to_vec(),
            decimals: params.ether_decimals,
            is_frozen: false,
        });

    return (force_register, metadata);
}
