use crate::{
    constants::*, Context, GatewayOperatingModeEnum,
    UpgradeArgs, PricingParametersArgs, GatewayAddressArgs,
    GatewayOperatingModeArgs, ForceCheckpointArgs
};
use alloy_primitives::{U256, utils::format_units};
use bridge_hub_rococo_runtime::runtime_types::snowbridge_pallet_ethereum_client;
use std::{fs::File, io::Read};
use sp_arithmetic::FixedU128;
use sp_crypto_hashing::twox_128;
use subxt::utils::Static;
use codec::Encode;

type CheckpointUpdate = snowbridge_beacon_primitives::CheckpointUpdate<512>;

use crate::asset_hub_runtime::runtime_types::asset_hub_rococo_runtime::RuntimeCall as AssetHubRuntimeCall;

use crate::bridge_hub_runtime::runtime_types::{
    bridge_hub_rococo_runtime::RuntimeCall as BridgeHubRuntimeCall,
    snowbridge_core::{
        outbound::v1::{Initializer, OperatingMode},
        pricing::{PricingParameters, Rewards},
    },
    snowbridge_pallet_system,
    sp_weights::weight_v2::Weight,
};

pub fn gateway_operating_mode(params: &GatewayOperatingModeArgs) -> BridgeHubRuntimeCall {
    let mode = match params.gateway_operating_mode {
        GatewayOperatingModeEnum::Normal => OperatingMode::Normal,
        GatewayOperatingModeEnum::RejectingOutboundMessages => {
            OperatingMode::RejectingOutboundMessages
        }
    };
    BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::set_operating_mode { mode },
    )
}

pub fn upgrade(params: &UpgradeArgs) -> BridgeHubRuntimeCall {
    let initializer = if params.initializer {
        Some((params.initializer_params.as_ref().unwrap().clone(), params.initializer_gas.unwrap()))
    } else {
        None
    };

    BridgeHubRuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::upgrade {
        impl_address: params.logic_address.into_array().into(),
        impl_code_hash: params.logic_code_hash.0.into(),
        initializer: initializer.map(|(params, gas)| Initializer {
            params: params.into(),
            maximum_required_gas: gas,
        }),
    })
}

pub async fn pricing_parameters(
    context: &Context,
    params: &PricingParametersArgs,
) -> Result<(BridgeHubRuntimeCall, AssetHubRuntimeCall), Box<dyn std::error::Error>> {

    // Calculate total outbound fee in BridgeHub
    let runtime_api_call = crate::bridge_hub_runtime::apis()
        .transaction_payment_call_api().query_weight_to_fee(Weight {
            ref_time: PROCESS_MESSAGE_WEIGHT.0 + COMMIT_SINGLE_MESSAGE_WEIGHT.0,
            proof_size: PROCESS_MESSAGE_WEIGHT.1 + COMMIT_SINGLE_MESSAGE_WEIGHT.1,
        });

    let local_fee = context.api
        .runtime_api()
        .at_latest()
        .await?
        .call(runtime_api_call)
        .await?;

    let remote_fee = crate::fees::calculate_remote_fee(
        FixedU128::from_rational(
            params.exchange_rate_numerator.into(),
            params.exchange_rate_denominator.into(),
        ),
        params.fee_per_gas,
        params.remote_reward,
    );

    let total_outbound_fee = local_fee.saturating_add(remote_fee);

    // Adjust outbound fee up by 10% as a buffer
    let total_outbound_fee_adjusted = total_outbound_fee.saturating_add(total_outbound_fee / 10);

    eprintln!("BridgeHub:");
    eprintln!(
        "  ExchangeRate: {} ETH/{}",
        params.exchange_rate_numerator as f64 / params.exchange_rate_denominator as f64,
        POLKADOT_SYMBOL
    );
    eprintln!(
        "  FeePerGas: {} GWEI",
        format_units(params.fee_per_gas, "gwei").unwrap(),
    );
    eprintln!(
        "  LocalReward: {} {} [{} PLANCK]",
        format_units(U256::from(params.local_reward), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        params.local_reward,
    );
    eprintln!(
        "  RemoteReward: {} ETH [{} WEI]",
        format_units(params.remote_reward, "eth").unwrap(),
        params.remote_reward
    );
    eprintln!("AssetHub:");
    eprintln!(
        "  BaseFee: {} {}, [{} PLANCK]",
        format_units(U256::from(total_outbound_fee_adjusted), POLKADOT_DECIMALS).unwrap(),
        POLKADOT_SYMBOL,
        total_outbound_fee_adjusted
    );

    // BridgeHub parameters
    let params: PricingParameters<u128> = PricingParameters {
        exchange_rate: Static(FixedU128::from_rational(
            params.exchange_rate_numerator.into(),
            params.exchange_rate_denominator.into(),
        )),
        fee_per_gas: bridge_hub_rococo_runtime::runtime_types::primitive_types::U256(
            U256::from(GWEI_UNIT)
                .checked_mul(U256::from(params.fee_per_gas))
                .unwrap()
                .into_limbs(),
        ),
        rewards: Rewards {
            local: params.local_reward.to::<u128>(),
            remote: bridge_hub_rococo_runtime::runtime_types::primitive_types::U256(
                params.remote_reward.into_limbs(),
            ),
        },
    };

    // AssetHub parameters
    let asset_hub_outbound_fee_storage_key: Vec<u8> = twox_128(b":BridgeHubEthereumBaseFee:").to_vec();
    let asset_hub_outbound_fee_encoded: Vec<u8> = total_outbound_fee_adjusted.encode();

    Ok((
        BridgeHubRuntimeCall::EthereumSystem(
            snowbridge_pallet_system::pallet::Call::set_pricing_parameters { params },
        ),
        AssetHubRuntimeCall::System(
            crate::asset_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
                items: vec![(asset_hub_outbound_fee_storage_key, asset_hub_outbound_fee_encoded)],
            },
        )
    ))
}

pub fn force_checkpoint(params: &ForceCheckpointArgs) -> BridgeHubRuntimeCall {
    let mut file = File::open(params.checkpoint.clone()).expect("File not found");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Failed to read the file");
    let checkpoint: CheckpointUpdate = serde_json::from_str(&data).unwrap();
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::force_checkpoint {
            update: Box::new(Static(checkpoint)),
        },
    )
}

pub fn set_gateway_address(params: &GatewayAddressArgs) -> BridgeHubRuntimeCall {
    let storage_key = sp_crypto_hashing::twox_128(b":EthereumGatewayAddress:").to_vec();
    let storage_value = params.gateway_address.into_array().encode();
    BridgeHubRuntimeCall::System(
        crate::bridge_hub_runtime::runtime_types::frame_system::pallet::Call::set_storage {
            items: vec![(storage_key, storage_value)],
        },
    )
}
