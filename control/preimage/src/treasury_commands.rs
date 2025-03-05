use crate::{constants::*, TreasuryProposal2024Args};

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

// USDC has 6 decimal places
pub const USDC_UNITS: u128 = 1_000_000;

#[derive(Copy, Clone)]
enum TreasuryAsset {
    DOT(u128),
    USDC(u128),
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
        }
    }
}

struct Spend {
    name: &'static str,
    asset: TreasuryAsset,
    delay: Option<u32>,
}

const SPENDS: [Spend; 23] = [
    Spend {
        name: "Operational & Development costs",
        asset: TreasuryAsset::DOT(187849),
        delay: None,
    },
    Spend {
        name: "Launch reward",
        asset: TreasuryAsset::DOT(158656),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #1",
        asset: TreasuryAsset::DOT(13221),
        delay: None,
    },
    Spend {
        name: "Milestone completion reward #2",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(30 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #3",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(60 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #4",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #5",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #6",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #7",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #8",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #9",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #10",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #11",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #12",
        asset: TreasuryAsset::DOT(13221),
        delay: Some(330 * DAYS),
    },
    Spend {
        name: "General Incentive reward #1",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "General Incentive reward #2",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "General Incentive reward #3",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "General Incentive reward #4",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "General Incentive reward #5",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "General Incentive reward #6",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "General Incentive reward #7",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "General Incentive reward #8",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "General Incentive reward #9",
        asset: TreasuryAsset::USDC(312500),
        delay: Some(330 * DAYS),
    },
];

pub const LAUNCH_BLOCK: u32 = 21292000;

pub fn treasury_proposal(params: &TreasuryProposal2024Args) -> RelayRuntimeCall {
    let mut calls: Vec<RelayRuntimeCall> = vec![];

    for spend in SPENDS.iter() {
        let (asset_id, asset_location, asset_amount) = spend.asset.into();
        let call = make_treasury_spend(
            params.beneficiary.into(),
            asset_location,
            asset_amount,
            spend.delay.map(|delay| LAUNCH_BLOCK + delay),
        );
        calls.push(call);
        println!("Spend: {}, {}({})", spend.name, asset_id, asset_amount);
    }

    utility_force_batch(calls)
}

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
