use crate::{
    constants::*, Context
};
use alloy_primitives::{utils::format_units, U256};
use codec::Encode;
use xcm_builder::GlobalConsensusParachainConvertsFor;
use sp_arithmetic::FixedU128;
use sp_crypto_hashing::twox_128;
use std::{fs::File, io::Read};
use subxt::utils::MultiAddress;
use subxt::utils::Static;

use crate::asset_hub_runtime::runtime_types::pallet_assets;
use crate::asset_hub_runtime::RuntimeCall as AssetHubRuntimeCall;

use crate::bridge_hub_runtime::RuntimeCall as BridgeHubRuntimeCall;

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

fn register_erc20(
    contract: [u8; 20],
    is_sufficient: bool,
    min_balance: u128,
) -> AssetHubRuntimeCall {
    use subxt::utils::AccountId32;
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;

    let asset_id = get_asset_id(chain_id, contract);
    // todo AHP
    let owner = GlobalConsensusParachainConvertsFor::<[u8; 32]>::from_chain_id(&chain_id);

    let force_register =
        AssetHubRuntimeCall::ForeignAssets(pallet_assets::pallet::Call2::force_create {
            id: asset_id,
            min_balance: params.ether_min_balance,
            is_sufficient,
            owner: MultiAddress::<AccountId32, ()>::Id(owner.into()),
        });

    return force_register;
}
