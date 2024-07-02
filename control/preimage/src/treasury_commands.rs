use crate::{constants::*, Context, TreasuryProposal2024Args};

use crate::helpers::utility_force_batch;
use crate::relay_runtime::api::runtime_types::{
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

pub const USDC_UNITS: u8 = 1000000;

#[derive(Copy, Clone)]
enum TreasuryAsset {
    DOT,
    USDC,
}

impl Into<Location> for TreasuryAsset {
    fn into(self) -> Location {
        match self {
            TreasuryAsset::DOT => Location {
                parents: 1,
                interior: Junctions::Here,
            },
            TreasuryAsset::USDC => Location {
                parents: 0,
                interior: Junctions::X2([
                    Junction::PalletInstance(50),
                    Junction::GeneralIndex(1337),
                ]),
            },
        }
    }
}

struct Spend {
    name: &'static str,
    asset: TreasuryAsset,
    amount: u128,
    delay: Option<u32>,
}

pub const fn dot(amount: u128) -> u128 {
    amount * UNITS
}

pub const fn usdc(amount: u128) -> u128 {
    amount * 1_000_000
}

pub const SPENDS: [Spend; 23] = [
    Spend {
        name: "Operational & Development costs",
        asset: TreasuryAsset::DOT,
        amount: dot(179611),
        delay: None,
    },
    Spend {
        name: "Launch reward",
        asset: TreasuryAsset::DOT,
        amount: dot(151699),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #1",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: None,
    },
    Spend {
        name: "Milestone completion reward #2",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(30 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #3",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(60 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #4",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #5",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #6",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #7",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #8",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #9",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #10",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #11",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "Milestone completion reward #12",
        asset: TreasuryAsset::DOT,
        amount: dot(12641),
        delay: Some(330 * DAYS),
    },
    Spend {
        name: "General Incentive reward #1",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(90 * DAYS),
    },
    Spend {
        name: "General Incentive reward #2",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(120 * DAYS),
    },
    Spend {
        name: "General Incentive reward #3",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(150 * DAYS),
    },
    Spend {
        name: "General Incentive reward #4",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(180 * DAYS),
    },
    Spend {
        name: "General Incentive reward #5",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(210 * DAYS),
    },
    Spend {
        name: "General Incentive reward #6",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(240 * DAYS),
    },
    Spend {
        name: "General Incentive reward #7",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(270 * DAYS),
    },
    Spend {
        name: "General Incentive reward #8",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(300 * DAYS),
    },
    Spend {
        name: "General Incentive reward #9",
        asset: TreasuryAsset::USDC,
        amount: usdc(312500),
        delay: Some(330 * DAYS),
    },
];

pub const LAUNCH_BLOCK: u32 = 21292000;

pub async fn treasury_proposal(
    ctx: &Context,
    params: TreasuryProposal2024Args,
) -> Result<RelayRuntimeCall, Box<dyn std::error::Error>> {

    let mut calls: Vec<RelayRuntimeCall> = vec![];

    for spend in SPENDS.iter() {
        let call = make_treasury_spend(
            params.beneficiary.into(),
            spend.asset,
            spend.amount,
            spend.delay.map(|delay| LAUNCH_BLOCK + delay),
        )
    }

    Ok(utility_force_batch(calls))
}

pub fn make_treasury_spend(
    beneficiary: [u8; 32],
    asset: TreasuryAsset,
    amount: u128,
    valid_from: Option<u32>,
) -> RelayRuntimeCall {
    let call = RelayRuntimeCall::Treasury(pallet_treasury::pallet::Call::spend {
        asset_kind: Box::new(VersionedLocatableAsset::V4 {
            location: Location {
                parents: 0,
                interior: Junctions::X1([Junction::Parachain(ASSET_HUB_ID)]),
            },
            asset_id: AssetId(asset.into()),
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
