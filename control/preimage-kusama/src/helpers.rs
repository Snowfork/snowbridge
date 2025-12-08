use codec::Encode;
use subxt::utils::Encoded;
use subxt::{OnlineClient, PolkadotConfig};

use crate::constants::ASSET_HUB_ID;
use crate::Context;

use crate::relay_runtime::runtime_types::{
    pallet_xcm,
    sp_weights::weight_v2::Weight,
    staging_xcm::v4::{
        junction::Junction,
        junctions::Junctions,
        location::Location,
        Instruction::{self, *},
        Xcm,
    },
    xcm::double_encoded::DoubleEncoded,
    xcm::v3::{MaybeErrorCode, OriginKind, WeightLimit},
    xcm::{VersionedLocation, VersionedXcm},
};

use crate::asset_hub_runtime::RuntimeCall as AssetHubRuntimeCall;
// Using the correct path for kusama runtime types
use crate::relay_runtime::runtime_types::staging_kusama_runtime::RuntimeCall as RelayRuntimeCall;

use sp_arithmetic::helpers_128bit::multiply_by_rational_with_rounding;
use sp_arithmetic::per_things::Rounding;

const MAX_REF_TIME: u128 = 500_000_000_000 - 1;
const MAX_PROOF_SIZE: u128 = 3 * 1024 * 1024 - 1;

// Increase call weight by 100% as a buffer in case the chain is upgraded with new weights
// while the proposal is still in flight.
pub fn increase_weight(ref_time: &mut u64, proof_size: &mut u64) {
    let _ref_time = multiply_by_rational_with_rounding(*ref_time as u128, 2, 1, Rounding::Up)
        .expect("overflow")
        .min(MAX_REF_TIME);
    let _proof_size = multiply_by_rational_with_rounding(*proof_size as u128, 2, 1, Rounding::Up)
        .expect("overflow")
        .min(MAX_PROOF_SIZE);

    *ref_time = _ref_time.try_into().expect("overflow");
    *proof_size = _proof_size.try_into().expect("overflow");
}

pub async fn send_xcm_asset_hub(
    context: &Context,
    calls: Vec<AssetHubRuntimeCall>,
) -> Result<RelayRuntimeCall, Box<dyn std::error::Error>> {
    let mut accum: Vec<(u64, u64, Vec<u8>)> = vec![];

    for call in calls.iter() {
        let (mut ref_time, mut proof_size) =
            query_weight_asset_hub(&context.asset_hub_api, call.clone()).await?;
        increase_weight(&mut ref_time, &mut proof_size);
        accum.push((ref_time, proof_size, call.encode()));
    }

    let mut instructions: Vec<Instruction> = vec![UnpaidExecution {
        weight_limit: WeightLimit::Unlimited,
        check_origin: None,
    }];

    for (ref_time, proof_size, encoded) in accum.into_iter() {
        instructions.append(&mut vec![
            Transact {
                origin_kind: OriginKind::Superuser,
                require_weight_at_most: Weight {
                    ref_time: ref_time,
                    proof_size: proof_size,
                },
                call: DoubleEncoded { encoded },
            },
            ExpectTransactStatus(MaybeErrorCode::Success),
        ]);
    }

    let call = RelayRuntimeCall::XcmPallet(pallet_xcm::pallet::Call::send {
        dest: Box::new(VersionedLocation::V4(Location {
            parents: 0,
            interior: Junctions::X1([Junction::Parachain(ASSET_HUB_ID)]),
        })),
        message: Box::new(VersionedXcm::V4(Xcm(instructions))),
    });

    Ok(call)
}

pub async fn query_weight_asset_hub(
    api: &OnlineClient<PolkadotConfig>,
    call: AssetHubRuntimeCall,
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let runtime_api_call = crate::asset_hub_runtime::apis()
        .transaction_payment_call_api()
        .query_call_info(call, 0);
    let call_info = api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;
    Ok((call_info.weight.ref_time, call_info.weight.proof_size))
}
