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
        pricing::{PricingParameters, Rewards},
    },
    snowbridge_outbound_queue_primitives::{v1::message::Initializer, OperatingMode},
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
    pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v3::multilocation::MultiLocation as Location;
    pub use crate::asset_hub_runtime::runtime_types::xcm::v3::{
        junction::Junction::AccountKey20,
        junction::Junction::GlobalConsensus,
        junction::NetworkId,
        junctions::Junctions::{X1, X2},
    };

    pub fn get_ether_id(chain_id: u64) -> Location {
        return Location {
            parents: 2,
            interior: X1(GlobalConsensus(NetworkId::Ethereum { chain_id })),
        };
    }
    pub fn get_asset_id(chain_id: u64, key: [u8; 20]) -> Location {
        return Location {
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

#[cfg(feature = "polkadot")]
fn register_polkadot_native_asset(
    location: crate::bridge_hub_runtime::runtime_types::xcm::VersionedLocation,
    name: &'static str,
    symbol: &'static str,
    decimals: u8,
) -> BridgeHubRuntimeCall {
    use crate::bridge_hub_runtime::runtime_types::{bounded_collections, snowbridge_core};

    let call = BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::register_token {
            location: location.into(),
            metadata: snowbridge_core::AssetMetadata {
                name: bounded_collections::bounded_vec::BoundedVec(name.as_bytes().to_vec()),
                symbol: bounded_collections::bounded_vec::BoundedVec(symbol.as_bytes().to_vec()),
                decimals,
            },
        },
    );

    return call;
}

#[cfg(feature = "polkadot")]
pub fn register_erc20_token_metadata() -> Vec<AssetHubRuntimeCall> {
    use hex_literal::hex;
    use alloy_primitives::Address;

    let tokens = vec![
        (hex!("9d39a5de30e57443bff2a8307a4256c8797a3497"), "Staked USDe", "sUSDe", 18),
        (hex!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"), "Wrapped Ether", "WETH", 18),
        (hex!("6982508145454ce325ddbe47a25d4ec3d2311933"), "Pepe", "PEPE", 18),
        (hex!("5a98fcbea516cf06857215779fd812ca3bef1b32"), "Lido DAO Token", "LDO", 18),
        (hex!("a3931d71877c0e7a3148cb7eb4463524fec27fbd"), "Savings USDS", "sUSDS", 18),
        (hex!("8236a87084f8b84306f72007f36f2618a5634494"), "Lombard Staked Bitcoin", "LBTC", 8),
        (hex!("1abaea1f7c830bd89acc67ec4af516284b1bc33c"), "Euro Coin", "EURC", 6),
        (hex!("56072c95faa701256059aa122697b133aded9279"), "SKY Governance Token", "SKY", 18),
        (hex!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), "USD Coin", "USDC", 6),
        (hex!("ba41ddf06b7ffd89d1267b5a93bfef2424eb2003"), "Mythos", "MYTH", 18),
        (hex!("0e186357c323c806c1efdad36d217f7a54b63d18"), "Curio Gas Token", "CGT2.0", 18),
        (hex!("aa7a9ca87d3694b5755f213b5d04094b8d0f0a6f"), "OriginTrail TRAC", "TRAC", 18),
        (hex!("18084fba666a33d37592fa2633fd49a74dd93a88"), "tBTC v2", "tBTC", 18),
        (hex!("7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0"), "Wrapped liquid staked Ether 2.0", "wstETH", 18),
        (hex!("582d872a1b094fc48f5de31d3b73f2d9be47def1"), "Wrapped TON Coin", "TONCOIN", 9),
        (hex!("6b175474e89094c44da98b954eedeac495271d0f"), "Dai Stablecoin", "DAI", 18),
        (hex!("95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce"), "SHIBA INU", "SHIB", 18),
        (hex!("7de91b204c1c737bcee6f000aaa6569cf7061cb7"), "Robonomics", "XRT", 9),
        (hex!("2260fac5e5542a773aa44fbcfedf7c193bc2c599"), "Wrapped BTC", "WBTC", 8),
        (hex!("8daebade922df735c38c80c7ebd708af50815faa"), "tBTC", "TBTC", 18),
        (hex!("5d3d01fd6d2ad1169b17918eb4f153c6616288eb"), "KILT", "KILT", 15),
        (hex!("dac17f958d2ee523a2206206994597c13d831ec7"), "Tether USD", "USDT", 6),
        (hex!("514910771af9ca656af840dff83e8264ecf986ca"), "ChainLink Token", "LINK", 18),
        (hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9"), "Aave Token", "AAVE", 18),
    ];

    tokens
        .into_iter()
        .map(|(contract_address, name, symbol, decimals)| {
            let params = UpdateAssetArgs {
                contract_id: Address::from(contract_address),
                name: name.to_string(),
                symbol: symbol.to_string(),
                decimals,
                min_balance: 1,
                is_sufficient: false,
                is_frozen: false,
            };
            force_set_metadata(&params)
        })
        .collect()
}

#[cfg(feature = "polkadot")]
pub fn token_registrations() -> Vec<BridgeHubRuntimeCall> {
    use crate::bridge_hub_runtime::runtime_types::{
        staging_xcm::v4::{
            junction::Junction::*, junction::NetworkId::*, junctions::Junctions::*,
            location::Location,
        },
        xcm::VersionedLocation,
    };
    use hex_literal::hex;
    return vec![
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: Here,
            }),
            "Polkadot",
            "DOT",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 2,
                interior: X1([GlobalConsensus(Kusama)]),
            }),
            "Kusama",
            "KSM",
            12u8,
        ),
        /*
         * Parachains
         */
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([Parachain(2004), PalletInstance(10)]),
            }),
            "Glimmer",
            "GLMR",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2030),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0001000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Bifrost Native Token",
            "BNC",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2030),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0900000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Voucher DOT",
            "vDOT",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([Parachain(2034), GeneralIndex(0)]),
            }),
            "Hydration",
            "HDX",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2039)]),
            }),
            "Integritee TEER",
            "TEER",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2051)]),
            }),
            "Ajuna Polkadot AJUN",
            "AJUN",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(3344)]),
            }),
            "Polimec",
            "PLMC",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(3370)]),
            }),
            "LAOS",
            "LAOS",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2086)]),
            }),
            "KILT Spiritnet",
            "KILT",
            15u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2006)]),
            }),
            "Astar",
            "ASTR",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2031),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0001000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Centrifuge",
            "CFG",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2101)]),
            }),
            "Subsocial",
            "SUB",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2035)]),
            }),
            "Phala Token",
            "PHA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2012),
                    GeneralKey {
                        length: 4,
                        data: hex!(
                            "5041524100000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Parallel",
            "PARA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2008)]),
            }),
            "Crust Parachain Native Token",
            "CRU",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X1([Parachain(2104)]),
            }),
            "Manta",
            "MANTA",
            18u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2000),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0000000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Acala",
            "ACA",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X2([
                    Parachain(2000),
                    GeneralKey {
                        length: 2,
                        data: hex!(
                            "0003000000000000000000000000000000000000000000000000000000000000"
                        ),
                    },
                ]),
            }),
            "Liquid DOT",
            "LDOT",
            10u8,
        ),
        /*
         * Meme coins
         */
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(30)]),
            }),
            "DED",
            "DED",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(23)]),
            }),
            "PINK",
            "PINK",
            10u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(86)]),
            }),
            "Kolkadot",
            "KOL",
            12u8,
        ),
        register_polkadot_native_asset(
            VersionedLocation::V4(Location {
                parents: 1,
                interior: X3([Parachain(1000), PalletInstance(50), GeneralIndex(31337)]),
            }),
            "GAVUN WUD",
            "WUD",
            10u8,
        ),
    ];
}
