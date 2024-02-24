// use bridge_hub_rococo_runtime::RuntimeCall;
use codec::Encode;
//use snowbridge_core::outbound::OperatingMode;

use clap::{Subcommand, ValueEnum};

use super::Context;

use subxt::{subxt, OnlineClient, PolkadotConfig};

#[cfg_attr(
    feature = "rococo",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "kusama",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "polkadot",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
pub mod relay_metadata {}

#[cfg_attr(
    feature = "rococo",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "kusama",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "polkadot",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
pub mod bridge_hub_metadata {}

use bridge_hub_metadata::runtime_types::{
    snowbridge_core::outbound::v1::OperatingMode,
    bridge_hub_rococo_runtime::RuntimeCall,
    snowbridge_pallet_system,
};

use relay_metadata::runtime_types::{
    polkadot_runtime::RuntimeCall as PolkadotRuntimeCall,
    sp_weights::weight_v2::Weight,
    xcm::{
        double_encoded::DoubleEncoded,
        v2::OriginKind,
        v3::{Instruction::*, junction::Junction, junctions::Junctions, WeightLimit, Xcm},
        VersionedXcm, VersionedMultiLocation,
    },
    staging_xcm::v3::multilocation::MultiLocation,
    pallet_xcm,
};

// use xcm::v3::prelude::*;

#[derive(Debug, Subcommand)]
pub enum GovernanceCommand {
    /// does testing things
    GatewayOperatingMode {
        #[arg(value_enum)]
        mode: Mode,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
pub enum Mode {
    Normal,
    RejectingOutboundMessages,
}

pub async fn run(context: &Context, command: &GovernanceCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        GovernanceCommand::GatewayOperatingMode { mode } => {
            let mode = match mode {
                Mode::Normal => OperatingMode::Normal,
                Mode::RejectingOutboundMessages => OperatingMode::RejectingOutboundMessages,
            };

            let call = RuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::set_operating_mode { mode });
            let encoded = call.encode();

            let (ref_time, proof_size) = query_weight(&context.bridge_hub_api, call).await?;

            let message = Box::new(VersionedXcm::V3(Xcm(vec![
                UnpaidExecution {
                    weight_limit: WeightLimit::Limited(Weight {
                        ref_time: ref_time,
                        proof_size: proof_size,
                    }),
                    check_origin: None,
                },
                Transact {
                    origin_kind: OriginKind::Xcm,
                    require_weight_at_most: Weight {
                        ref_time: ref_time,
                        proof_size: proof_size,
                    },
                    call: DoubleEncoded { encoded },
                },
            ])));

            let para_id = query_para_id(&context.bridge_hub_api).await?;

            let dest = Box::new(VersionedMultiLocation::V3(MultiLocation {
                parents: 0,
                interior: Junctions::X1(Junction::Parachain(para_id)),
            }));

            let call2 = PolkadotRuntimeCall::XcmPallet(pallet_xcm::pallet::Call::send { dest, message });

            let encoded2 = call2.encode();

            println!("0x{}", hex::encode(encoded2));
        }
    };


    Ok(())
}


pub async fn query_weight(api: &OnlineClient<PolkadotConfig>, call: RuntimeCall) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let runtime_api_call = bridge_hub_metadata::apis().transaction_payment_call_api().query_call_info(call, 0);
    let call_info = api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;
    Ok((call_info.weight.ref_time, call_info.weight.proof_size))
}

pub async fn query_para_id(api: &OnlineClient<PolkadotConfig>) -> Result<u32, Box<dyn std::error::Error>> {
    let storage_query = bridge_hub_metadata::storage().parachain_info().parachain_id();
    let bridge_hub_para_id = api
        .storage()
        .at_latest()
        .await?
        .fetch(&storage_query)
        .await?
        .expect("parachain id not set");

    Ok(bridge_hub_para_id.0)
}
