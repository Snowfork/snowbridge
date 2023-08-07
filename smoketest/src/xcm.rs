use crate::constants::*;
use crate::parachains::assethub::{
    api::runtime_types as assetHubTypes, api::runtime_types::xcm as assetHubXcm,
};
use assetHubTypes::sp_weights::weight_v2::Weight;
use assetHubXcm::{
    double_encoded::DoubleEncoded,
    v2::OriginKind,
    v3::{
        junctions::Junctions,
        multiasset::{AssetId::Concrete, Fungibility::Fungible, MultiAsset, MultiAssets},
        multilocation::MultiLocation,
        Instruction, WeightLimit, Xcm,
    },
    VersionedXcm,
};

pub fn construct_xcm_message(encoded_call: Vec<u8>) -> Box<VersionedXcm> {
    Box::new(VersionedXcm::V3(Xcm(vec![
        Instruction::UnpaidExecution {
            weight_limit: WeightLimit::Limited(Weight {
                ref_time: ASSET_HUB_WEIGHT_REQUIRED,
                proof_size: ASSET_HUB_PROOF_SIZE_REQUIRED,
            }),
            check_origin: None,
        },
        Instruction::Transact {
            origin_kind: OriginKind::Xcm,
            require_weight_at_most: Weight {
                ref_time: ASSET_HUB_WEIGHT_REQUIRED,
                proof_size: ASSET_HUB_PROOF_SIZE_REQUIRED,
            },
            call: DoubleEncoded {
                encoded: encoded_call,
            },
        },
    ])))
}

// WithdrawAsset is not allowed in bridgehub but keep it here
pub async fn construct_xcm_message_with_fee(encoded_call: Vec<u8>) -> Box<VersionedXcm> {
    let buy_execution_fee = MultiAsset {
        id: Concrete(MultiLocation {
            parents: 0,
            interior: Junctions::Here,
        }),
        fun: Fungible(BRIDGE_HUB_FEE_REQUIRED),
    };

    Box::new(VersionedXcm::V3(Xcm(vec![
        Instruction::WithdrawAsset(MultiAssets(vec![buy_execution_fee])),
        Instruction::BuyExecution {
            fees: MultiAsset {
                id: Concrete(MultiLocation {
                    parents: 0,
                    interior: Junctions::Here,
                }),
                fun: Fungible(BRIDGE_HUB_FEE_REQUIRED),
            },
            weight_limit: WeightLimit::Unlimited,
        },
        Instruction::Transact {
            origin_kind: OriginKind::Xcm,
            require_weight_at_most: Weight {
                ref_time: ASSET_HUB_WEIGHT_REQUIRED,
                proof_size: ASSET_HUB_PROOF_SIZE_REQUIRED,
            },
            call: DoubleEncoded {
                encoded: encoded_call,
            },
        },
    ])))
}
