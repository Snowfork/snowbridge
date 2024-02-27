use subxt::{OnlineClient, PolkadotConfig};
use codec::Encode;

use crate::Context;

use crate::bridge_hub_runtime::{
    self,
    runtime_types::bridge_hub_rococo_runtime::RuntimeCall as BridgeHubRuntimeCall,
};

use crate::relay_runtime::runtime_types::{
    polkadot_runtime::RuntimeCall as RelayRuntimeCall,
    sp_weights::weight_v2::Weight,
    xcm::{
        double_encoded::DoubleEncoded,
        v2::OriginKind,
        v3::{Instruction::{self, *}, junction::Junction, junctions::Junctions, WeightLimit, MaybeErrorCode, Xcm},
        VersionedXcm,
        VersionedMultiLocation
    },
    staging_xcm::v3::multilocation::MultiLocation,
    pallet_xcm
};

use sp_arithmetic::helpers_128bit::multiply_by_rational_with_rounding;
use sp_arithmetic::per_things::Rounding;

// Increase call weight by 10% as a buffer in case the chain is upgraded with new weights
// while the proposal is still in flight.
pub fn increase_weight(ref_time: &mut u64, proof_size: &mut u64) {
    let x = multiply_by_rational_with_rounding(*ref_time as u128, 110, 100, Rounding::Up).expect("overflow");
    let y = multiply_by_rational_with_rounding(*proof_size as u128, 110, 100, Rounding::Up).expect("overflow");

    *ref_time = x.try_into().expect("overflow");
    *proof_size = y.try_into().expect("overflow");
}

// Base weight costs for Transact and ExpectTransactStatus
const TRANSACT_INSTR_WEIGHT: u64 = 10_000_000;
const EXPECT_STATUS_WEIGHT: u64 = 10_000_000;

pub async fn wrap_calls(context: &Context, calls: Vec<BridgeHubRuntimeCall>) -> Result<RelayRuntimeCall, Box<dyn std::error::Error>> {
    let mut total_ref_time: u64 = 0;
    let mut total_proof_size: u64 = 0;
    let mut accum: Vec<(u64, u64, Vec<u8>)> = vec![];

    for call in calls.iter() {
        let (mut ref_time, mut proof_size) = query_weight(&context.api, call.clone()).await?;
        increase_weight(&mut ref_time, &mut proof_size);
        total_ref_time += ref_time + TRANSACT_INSTR_WEIGHT + EXPECT_STATUS_WEIGHT;
        total_proof_size += proof_size;
        accum.push((ref_time, proof_size, call.encode()));
    }

    let mut instructions: Vec<Instruction> = vec![
        UnpaidExecution {
            weight_limit: WeightLimit::Limited(Weight {
                ref_time: total_ref_time,
                proof_size: total_proof_size,
            }),
            check_origin: None,
        },
    ];

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

    let para_id = query_para_id(&context.api).await?;

    let call = RelayRuntimeCall::XcmPallet(pallet_xcm::pallet::Call::send {
        dest: Box::new(VersionedMultiLocation::V3(MultiLocation {
            parents: 0,
            interior: Junctions::X1(Junction::Parachain(para_id)),
        })),
        message: Box::new(VersionedXcm::V3(Xcm(instructions)))
    });

    Ok(call)
}

pub async fn query_weight(api: &OnlineClient<PolkadotConfig>, call: BridgeHubRuntimeCall) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let runtime_api_call = bridge_hub_runtime::apis().transaction_payment_call_api().query_call_info(call, 0);
    let call_info = api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;
    Ok((call_info.weight.ref_time, call_info.weight.proof_size))
}

pub async fn query_para_id(api: &OnlineClient<PolkadotConfig>) -> Result<u32, Box<dyn std::error::Error>> {
    let storage_query = bridge_hub_runtime::storage().parachain_info().parachain_id();
    let bridge_hub_para_id = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?
        .expect("parachain id not set");

    Ok(bridge_hub_para_id.0)
}
