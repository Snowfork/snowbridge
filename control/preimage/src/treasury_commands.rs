use crate::{constants::*, TreasuryProposal2025Args};

use crate::helpers::utility_force_batch;
use crate::relay_runtime::runtime_types::{
    pallet_treasury,
    polkadot_runtime_common::impls::VersionedLocatableAsset,
    staging_xcm::v4::{
        asset::AssetId, junction::Junction, junctions::Junctions, location::Location,
    },
    xcm::VersionedLocation,
};
use crate::relay_runtime::RuntimeCall as RelayRuntimeCall;
use polkadot_runtime_constants::currency::UNITS;
use polkadot_runtime_constants::time::DAYS;

// USDC and USDT has 6 decimal places
pub const USDC_UNITS: u128 = 1_000_000;
pub const USDT_UNITS: u128 = 1_000_000;

#[derive(Copy, Clone)]
enum TreasuryAsset {
    DOT(u128),
    USDC(u128),
    USDT(u128),
}

impl Into<(&str, Location, u128)> for TreasuryAsset {
    fn into(self) -> (&'static str, Location, u128) {
        match self {
            TreasuryAsset::DOT(value) => (
                "DOT",
                Location {
                    parents: 1,
                    interior: Junctions::Here,
                },
                value * UNITS,
            ),
            TreasuryAsset::USDC(value) => (
                "USDC",
                Location {
                    parents: 0,
                    interior: Junctions::X2([
                        Junction::PalletInstance(50),
                        Junction::GeneralIndex(1337),
                    ]),
                },
                value * USDC_UNITS,
            ),
            TreasuryAsset::USDT(value) => (
                "USDT",
                Location {
                    parents: 0,
                    interior: Junctions::X2([
                        Junction::PalletInstance(50),
                        Junction::GeneralIndex(1984),
                    ]),
                },
                value * USDT_UNITS,
            ),
        }
    }
}

struct Spend {
    name: &'static str,
    asset: TreasuryAsset,
    delay: Option<u32>,
}

const SPENDS: [Spend; 36] = [
    // DOT spends
    Spend {
        name: "2025 DOT spend #1",
        asset: TreasuryAsset::DOT(17800),
        delay: None,
    },
    Spend {
        name: "2025 DOT spend #2",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(30 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #3",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(60 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #4",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #5",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #6",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #7",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #8",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #9",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #10",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #11",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "2025 DOT spend #12",
        asset: TreasuryAsset::DOT(17800),
        delay: Some(330 * DAYS),
    },
    // USDC spends
    Spend {
        name: "2025 USDC spend #1",
        asset: TreasuryAsset::USDC(312500),
        delay: None,
    },
    Spend {
        name: "2025 USDC spend #2",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(30 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #3",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(60 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #4",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #5",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #6",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #7",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #8",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #9",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #10",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #11",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "2025 USDC spend #12",
        asset: TreasuryAsset::USDC(156250),
        delay: Some(330 * DAYS),
    },
    // USDT spends
    Spend {
        name: "2025 USDT spend #1",
        asset: TreasuryAsset::USDT(312500),
        delay: None,
    },
    Spend {
        name: "2025 USDT spend #2",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(30 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #3",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(60 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #4",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #5",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #6",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #7",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #8",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #9",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #10",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #11",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "2025 USDT spend #12",
        asset: TreasuryAsset::USDT(156250),
        delay: Some(330 * DAYS),
    },
];

fn make_treasury_spend(
    beneficiary: [u8; 32],
    asset: Location,
    amount: u128,
    valid_from: Option<u32>,
) -> RelayRuntimeCall {
    let call = RelayRuntimeCall::Treasury(pallet_treasury::pallet::Call::spend {
        asset_kind: Box::new(VersionedLocatableAsset::V4 {
            location: Location {
                parents: 0,
                interior: Junctions::X1([Junction::Parachain(ASSET_HUB_ID)]),
            },
            asset_id: AssetId(asset),
        }),
        beneficiary: Box::new(VersionedLocation::V4(Location {
            parents: 0,
            interior: Junctions::X1([Junction::AccountId32 {
                network: None,
                id: beneficiary,
            }]),
        })),
        amount,
        valid_from,
    });

    call
}

pub fn treasury_proposal_2025(params: &TreasuryProposal2025Args) -> RelayRuntimeCall {
    let mut calls: Vec<RelayRuntimeCall> = vec![];

    for spend in SPENDS.iter() {
        let (asset_id, asset_location, asset_amount) = spend.asset.into();
        let call = make_treasury_spend(
            params.beneficiary.into(),
            asset_location,
            asset_amount,
            spend.delay.map(|delay| params.base_block + delay),
        );
        calls.push(call);
        println!("Spend: {}, {}({})", spend.name, asset_id, asset_amount);
    }

    utility_force_batch(calls)
}
