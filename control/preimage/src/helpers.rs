use bridge_hub_runtime::ethereum_system::storage::types::pricing_parameters::PricingParameters;
use codec::Encode;
use subxt::{utils::H160, utils::H256, OnlineClient, PolkadotConfig};

use crate::constants::BRIDGE_HUB_ID;
use crate::Context;

use crate::bridge_hub_runtime::{self, RuntimeCall as BridgeHubRuntimeCall};

use crate::asset_hub_runtime::runtime_types::{
    pallet_xcm,
    sp_weights::weight_v2::Weight,
    staging_xcm::v5::{
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

use bridge_hub_runtime::runtime_types::snowbridge_outbound_queue_primitives::v1::message::{
    AgentExecuteCommand, Command, Fee,
};
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

pub async fn send_xcm_bridge_hub(
    context: &Context,
    calls: Vec<BridgeHubRuntimeCall>,
) -> Result<AssetHubRuntimeCall, Box<dyn std::error::Error>> {
    let mut accum: Vec<(u64, u64, Vec<u8>)> = vec![];

    for call in calls.iter() {
        let (mut ref_time, mut proof_size) =
            query_weight_bridge_hub(&context.bridge_hub_api, call.clone()).await?;
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
                fallback_max_weight: Some(Weight {
                    ref_time: ref_time,
                    proof_size: proof_size,
                }),
                call: DoubleEncoded { encoded },
            },
            ExpectTransactStatus(MaybeErrorCode::Success),
        ]);
    }

    let call = AssetHubRuntimeCall::PolkadotXcm(pallet_xcm::pallet::Call::send {
        dest: Box::new(VersionedLocation::V5(Location {
            parents: 1,
            interior: Junctions::X1([Junction::Parachain(BRIDGE_HUB_ID)]),
        })),
        message: Box::new(VersionedXcm::V5(Xcm(instructions))),
    });

    Ok(call)
}

pub async fn send_xcm_asset_hub(
    _context: &Context,
    calls: Vec<AssetHubRuntimeCall>,
) -> Result<AssetHubRuntimeCall, Box<dyn std::error::Error>> {
    if calls.len() == 1 {
        Ok(calls.into_iter().next().unwrap())
    } else {
        Ok(utility_force_batch(calls))
    }
}

pub async fn query_weight_bridge_hub(
    api: &OnlineClient<PolkadotConfig>,
    call: BridgeHubRuntimeCall,
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let runtime_api_call = bridge_hub_runtime::apis()
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

#[allow(dead_code)]
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

pub fn utility_force_batch(calls: Vec<AssetHubRuntimeCall>) -> AssetHubRuntimeCall {
    AssetHubRuntimeCall::Utility(
        crate::asset_hub_runtime::runtime_types::pallet_utility::pallet::Call::batch_all { calls },
    )
}

#[cfg(any(feature = "westend", feature = "paseo"))]
pub fn sudo(call: Box<AssetHubRuntimeCall>) -> AssetHubRuntimeCall {
    return AssetHubRuntimeCall::Sudo(
        crate::asset_hub_runtime::runtime_types::pallet_sudo::pallet::Call::sudo { call },
    );
}

pub fn force_xcm_version() -> AssetHubRuntimeCall {
    use crate::asset_hub_runtime::runtime_types::staging_xcm::v5::{
        junction::Junction::GlobalConsensus, junction::NetworkId, junctions::Junctions::X1,
        location::Location,
    };
    let chain_id = crate::bridge_hub_runtime::CHAIN_ID;
    let location = Box::new(Location {
        parents: 2,
        interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id })]),
    });

    AssetHubRuntimeCall::PolkadotXcm(
        crate::asset_hub_runtime::runtime_types::pallet_xcm::pallet::Call::force_xcm_version {
            location,
            version: 4,
        },
    )
}

pub async fn calculate_delivery_fee(
    api: &OnlineClient<PolkadotConfig>,
    params: &PricingParameters,
) -> Result<Fee<u128>, Box<dyn std::error::Error>> {
    let command = Command::AgentExecute {
        agent_id: H256::zero(),
        command: AgentExecuteCommand::TransferToken {
            token: H160::zero(),
            recipient: H160::zero(),
            amount: 0,
        },
    };
    let runtime_api_call = bridge_hub_runtime::apis()
        .outbound_queue_api()
        .calculate_fee(command, Some(params.clone()));
    let fee = api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;

    Ok(fee)
}
