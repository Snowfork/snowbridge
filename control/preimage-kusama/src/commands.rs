use crate::asset_hub_runtime::runtime_types::pallet_assets;
use crate::asset_hub_runtime::RuntimeCall as AssetHubRuntimeCall;
use subxt::utils::MultiAddress;

use crate::xcm_helper;

pub use crate::asset_hub_runtime::runtime_types::staging_xcm::v4::{
    junction::Junction::{AccountKey20, GeneralIndex, GlobalConsensus, PalletInstance, Parachain},
    junction::NetworkId,
    junctions::Junctions::{X1, X2, X4},
    location::Location,
};
use hex_literal::hex;
use subxt::utils::AccountId32;

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

fn register_erc20(
    contract: [u8; 20],
    is_sufficient: bool,
    min_balance: u128,
) -> AssetHubRuntimeCall {
    let chain_id = crate::constants::CHAIN_ID;

    let asset_id = get_asset_id(chain_id, contract);
    let owner = xcm_helper::get_pah_owner_on_kusama();

    let force_register =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_create {
            id: asset_id,
            min_balance,
            is_sufficient,
            owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        });

    return force_register;
}

pub fn register_asset_metadata(
    asset_id: Location,
    name: String,
    symbol: String,
    decimals: u8,
) -> AssetHubRuntimeCall {
    AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_set_metadata {
        id: asset_id,
        name: name.as_bytes().to_vec(),
        symbol: symbol.as_bytes().to_vec(),
        decimals,
        is_frozen: false,
    })
}

pub fn register_erc20_with_metadata(
    contract: [u8; 20],
    min_balance: u128,
    name: String,
    symbol: String,
    decimals: u8,
    is_sufficient: bool,
) -> (AssetHubRuntimeCall, AssetHubRuntimeCall) {
    let chain_id = crate::constants::CHAIN_ID;
    let asset_id = get_asset_id(chain_id, contract);

    register_asset_with_metadata(asset_id, min_balance, name, symbol, decimals, is_sufficient)
}

pub fn register_ether(
    ether_min_balance: u128,
    ether_name: String,
    ether_symbol: String,
    ether_decimals: u8,
    ether_sufficient: bool,
) -> (AssetHubRuntimeCall, AssetHubRuntimeCall) {
    let chain_id = crate::constants::CHAIN_ID;
    let asset_id = get_ether_id(chain_id);

    register_asset_with_metadata(
        asset_id,
        ether_min_balance,
        ether_name,
        ether_symbol,
        ether_decimals,
        ether_sufficient,
    )
}

pub fn register_asset_with_metadata(
    asset_id: Location,
    min_balance: u128,
    name: String,
    symbol: String,
    decimals: u8,
    is_sufficient: bool,
) -> (AssetHubRuntimeCall, AssetHubRuntimeCall) {
    let owner = xcm_helper::get_pah_owner_on_kusama();

    let force_register =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_create {
            id: asset_id.clone(),
            min_balance,
            is_sufficient,
            owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        });
    let metadata = register_asset_metadata(asset_id, name, symbol, decimals);

    return (force_register, metadata);
}

pub fn token_registrations() -> Vec<AssetHubRuntimeCall> {
    let mut calls = Vec::new();

    // Ether
    let (ether_create, ether_metadata) = register_ether(
        15_000_000_000_000,
        "Ether".to_string(),
        "ETH".to_string(),
        18,
        true,
    );
    calls.push(ether_create);
    calls.push(ether_metadata);

    // Wrapped Ether (WETH)
    let (weth_create, weth_metadata) = register_erc20_with_metadata(
        hex!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
        15_000_000_000_000,
        "Wrapped Ether".to_string(),
        "WETH".to_string(),
        18,
        true,
    );
    calls.push(weth_create);
    calls.push(weth_metadata);

    // USDT
    let (usdt_create, usdt_metadata) = register_erc20_with_metadata(
        hex!("dac17f958d2ee523a2206206994597c13d831ec7"),
        10_000,
        "USDT (Snowbridge)".to_string(),
        "USDT".to_string(),
        6,
        true,
    );
    calls.push(usdt_create);
    calls.push(usdt_metadata);

    // USDC
    let (usdc_create, usdc_metadata) = register_erc20_with_metadata(
        hex!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
        10_000,
        "USDC (Snowbridge)".to_string(),
        "USDC".to_string(),
        6,
        true,
    );

    calls.push(usdc_create);
    calls.push(usdc_metadata);

    // Staked USDe
    calls.push(register_erc20(
        hex!("9d39a5de30e57443bff2a8307a4256c8797a3497"),
        false,
        1,
    ));
    // Pepe
    calls.push(register_erc20(
        hex!("6982508145454ce325ddbe47a25d4ec3d2311933"),
        false,
        1,
    ));
    // LDO
    calls.push(register_erc20(
        hex!("5a98fcbea516cf06857215779fd812ca3bef1b32"),
        false,
        1,
    ));
    // Savings USDS
    calls.push(register_erc20(
        hex!("a3931d71877c0e7a3148cb7eb4463524fec27fbd"),
        false,
        1,
    ));
    // LBTC
    calls.push(register_erc20(
        hex!("8236a87084f8b84306f72007f36f2618a5634494"),
        false,
        1,
    ));
    // EurC
    calls.push(register_erc20(
        hex!("1abaea1f7c830bd89acc67ec4af516284b1bc33c"),
        false,
        1,
    ));
    // Sky
    calls.push(register_erc20(
        hex!("56072c95faa701256059aa122697b133aded9279"),
        false,
        1,
    ));
    // Myth
    calls.push(register_erc20(
        hex!("ba41ddf06b7ffd89d1267b5a93bfef2424eb2003"),
        false,
        1,
    ));
    // tBTC v2
    calls.push(register_erc20(
        hex!("18084fba666a33d37592fa2633fd49a74dd93a88"),
        false,
        1,
    ));
    // wstETH
    calls.push(register_erc20(
        hex!("7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0"),
        false,
        1,
    ));
    // TONCOIN
    calls.push(register_erc20(
        hex!("582d872a1b094fc48f5de31d3b73f2d9be47def1"),
        false,
        1,
    ));
    // DAI
    calls.push(register_erc20(
        hex!("6b175474e89094c44da98b954eedeac495271d0f"),
        false,
        1,
    ));
    // SHIB
    calls.push(register_erc20(
        hex!("95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce"),
        false,
        1,
    ));
    // WBTC
    calls.push(register_erc20(
        hex!("2260fac5e5542a773aa44fbcfedf7c193bc2c599"),
        false,
        1,
    ));
    // tBTC
    calls.push(register_erc20(
        hex!("8daebade922df735c38c80c7ebd708af50815faa"),
        false,
        1,
    ));
    // Kilt
    calls.push(register_erc20(
        hex!("5d3d01fd6d2ad1169b17918eb4f153c6616288eb"),
        false,
        1,
    ));
    // LINK
    calls.push(register_erc20(
        hex!("514910771af9ca656af840dff83e8264ecf986ca"),
        false,
        1,
    ));
    // AAVE
    calls.push(register_erc20(
        hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9"),
        false,
        1,
    ));
    // Curio
    calls.push(register_erc20(
        hex!("0e186357c323c806c1efdad36d217f7a54b63d18"),
        false,
        1,
    ));

    // Gavun WUD
    let (wud_create, wud_meta) = register_asset_with_metadata(
        Location {
            parents: 2,
            interior: X4([
                GlobalConsensus(NetworkId::Polkadot),
                Parachain(1000),
                PalletInstance(50),
                GeneralIndex(31337),
            ]),
        },
        10_000_000,
        "GAVUN WUD".to_string(),
        "WUD".to_string(),
        10,
        false,
    );
    calls.push(wud_create);
    calls.push(wud_meta);

    calls
}

pub fn register_erc20_token_metadata_and_wud() -> Vec<AssetHubRuntimeCall> {
    let chain_id = crate::constants::CHAIN_ID;
    let mut calls = Vec::new();

    // ERC-20 token metadata
    let tokens = vec![
        (
            hex!("9d39a5de30e57443bff2a8307a4256c8797a3497"),
            "Staked USDe",
            "sUSDe",
            18,
        ),
        (
            hex!("6982508145454ce325ddbe47a25d4ec3d2311933"),
            "Pepe",
            "PEPE",
            18,
        ),
        (
            hex!("5a98fcbea516cf06857215779fd812ca3bef1b32"),
            "Lido DAO Token",
            "LDO",
            18,
        ),
        (
            hex!("a3931d71877c0e7a3148cb7eb4463524fec27fbd"),
            "Savings USDS",
            "sUSDS",
            18,
        ),
        (
            hex!("8236a87084f8b84306f72007f36f2618a5634494"),
            "Lombard Staked Bitcoin",
            "LBTC",
            8,
        ),
        (
            hex!("1abaea1f7c830bd89acc67ec4af516284b1bc33c"),
            "Euro Coin",
            "EURC",
            6,
        ),
        (
            hex!("56072c95faa701256059aa122697b133aded9279"),
            "SKY Governance Token",
            "SKY",
            18,
        ),
        (
            hex!("ba41ddf06b7ffd89d1267b5a93bfef2424eb2003"),
            "Mythos",
            "MYTH",
            18,
        ),
        (
            hex!("18084fba666a33d37592fa2633fd49a74dd93a88"),
            "tBTC v2",
            "tBTC",
            18,
        ),
        (
            hex!("7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0"),
            "Wrapped liquid staked Ether 2.0",
            "wstETH",
            18,
        ),
        (
            hex!("582d872a1b094fc48f5de31d3b73f2d9be47def1"),
            "Wrapped TON Coin",
            "TONCOIN",
            9,
        ),
        (
            hex!("6b175474e89094c44da98b954eedeac495271d0f"),
            "Dai Stablecoin",
            "DAI",
            18,
        ),
        (
            hex!("95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce"),
            "SHIBA INU",
            "SHIB",
            18,
        ),
        (
            hex!("2260fac5e5542a773aa44fbcfedf7c193bc2c599"),
            "Wrapped BTC",
            "WBTC",
            8,
        ),
        (
            hex!("8daebade922df735c38c80c7ebd708af50815faa"),
            "tBTC",
            "TBTC",
            18,
        ),
        (
            hex!("5d3d01fd6d2ad1169b17918eb4f153c6616288eb"),
            "KILT",
            "KILT",
            15,
        ),
        (
            hex!("514910771af9ca656af840dff83e8264ecf986ca"),
            "ChainLink Token",
            "LINK",
            18,
        ),
        (
            hex!("7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9"),
            "Aave Token",
            "AAVE",
            18,
        ),
        (
            hex!("0e186357c323c806c1efdad36d217f7a54b63d18"),
            "Curio Gas Token",
            "CGT2.0",
            18,
        ),
    ];

    // Add ERC-20 metadata calls
    for (contract_address, name, symbol, decimals) in tokens {
        let asset_id = get_asset_id(chain_id, contract_address);
        calls.push(register_asset_metadata(
            asset_id,
            name.to_string(),
            symbol.to_string(),
            decimals,
        ));
    }

    // Add WUD token registration (both creation and metadata)
    let (wud_create, wud_meta) = register_asset_with_metadata(
        Location {
            parents: 2,
            interior: X4([
                GlobalConsensus(NetworkId::Polkadot),
                Parachain(1000),
                PalletInstance(50),
                GeneralIndex(31337),
            ]),
        },
        10_000_000,
        "GAVUN WUD".to_string(),
        "WUD".to_string(),
        10,
        false,
    );
    calls.push(wud_create);
    calls.push(wud_meta);

    calls
}
