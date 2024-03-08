use crate::GatewayOperatingModeEnum;

use alloy_primitives::{Address, Bytes, FixedBytes, U128, U256};
use bridge_hub_rococo_runtime::runtime_types::snowbridge_pallet_ethereum_client;
use snowbridge_beacon_primitives::CheckpointUpdate;
use sp_arithmetic::{FixedPointNumber, FixedU128};
use subxt::utils::{H160, H256};

use crate::bridge_hub_runtime::runtime_types::{
    bridge_hub_rococo_runtime::RuntimeCall as BridgeHubRuntimeCall,
    snowbridge_core::{
        outbound::v1::{Initializer, OperatingMode},
        pricing::{PricingParameters, Rewards},
    },
    snowbridge_pallet_system,
};

pub fn gateway_operating_mode(mode: GatewayOperatingModeEnum) -> BridgeHubRuntimeCall {
    let mode = match mode {
        GatewayOperatingModeEnum::Normal => OperatingMode::Normal,
        GatewayOperatingModeEnum::RejectingOutboundMessages => {
            OperatingMode::RejectingOutboundMessages
        }
    };
    BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::set_operating_mode { mode },
    )
}

pub fn upgrade(
    logic_address: Address,
    logic_code_hash: FixedBytes<32>,
    initializer: Option<(Bytes, u64)>,
) -> BridgeHubRuntimeCall {
    BridgeHubRuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::upgrade {
        impl_address: H160::from_slice(logic_address.as_slice()),
        impl_code_hash: H256::from_slice(logic_code_hash.as_slice()),
        initializer: initializer.map(|(params, gas)| Initializer {
            params: params.into(),
            maximum_required_gas: gas,
        }),
    })
}

#[cfg(feature = "rococo")]
const POLKADOT_DECIMALS: u128 = 1_000_000_000_000;

#[cfg(feature = "kusama")]
const POLKADOT_DECIMALS: u128 = 1_000_000_000_000;

#[cfg(feature = "polkadot")]
const POLKADOT_DECIMALS: u128 = 10_000_000_000;

const ETHER_DECIMALS: u128 = 1_000_000_000_000_000_000;

const GWEI_DECIMALS: u128 = 1_000_000_000;

#[cfg(feature = "rococo")]
const POLKADOT_SYMBOL: &str = "ROC";

#[cfg(feature = "kusama")]
const POLKADOT_SYMBOL: &str = "KSM";

#[cfg(feature = "polkadot")]
const POLKADOT_SYMBOL: &str = "DOT";

fn format_rational(fixed: FixedU128) -> String {
    let inner = fixed.into_inner();
    let integral = {
        let int = inner / FixedU128::accuracy();
        let signum_for_zero = if int == 0 && fixed.is_negative() {
            "-"
        } else {
            ""
        };
        format!("{}{}", signum_for_zero, int)
    };
    let precision = (FixedU128::accuracy() as f64).log10() as usize;
    let fractional = format!(
        "{:0>weight$}",
        ((inner % FixedU128::accuracy()) as i128).abs(),
        weight = precision
    );

    format!("{}.{}", integral, fractional)
}

pub fn pricing_parameters(
    exchange_rate_numerator: u64,
    exchange_rate_denominator: u64,
    fee_per_gas: u64,
    local_reward: U128,
    remote_reward: U256,
) -> BridgeHubRuntimeCall {
    eprintln!("Pricing Parameters:");
    eprintln!(
        "  ExchangeRate: {} ETH/{}",
        format_rational(FixedU128::from_rational(
            exchange_rate_numerator.into(),
            exchange_rate_denominator.into()
        )),
        POLKADOT_SYMBOL
    );
    eprintln!(
        "  FeePerGas: {} GWEI",
        format_rational(FixedU128::from_rational(
            fee_per_gas as u128 * GWEI_DECIMALS,
            GWEI_DECIMALS
        ))
    );
    eprintln!(
        "  LocalReward: {} {}",
        format_rational(FixedU128::from_rational(
            local_reward.to::<u128>(),
            POLKADOT_DECIMALS
        )),
        POLKADOT_SYMBOL
    );
    eprintln!(
        "  RemoteReward: {} ETH",
        format_rational(FixedU128::from_rational(
            remote_reward.to::<u128>(),
            ETHER_DECIMALS
        ))
    );

    let params: PricingParameters<u128> = PricingParameters {
        exchange_rate:
            bridge_hub_rococo_runtime::runtime_types::sp_arithmetic::fixed_point::FixedU128(
                FixedU128::from_rational(
                    exchange_rate_numerator.into(),
                    exchange_rate_denominator.into(),
                )
                .into_inner(),
            ),
        fee_per_gas: bridge_hub_rococo_runtime::runtime_types::primitive_types::U256(
            U256::from(GWEI_DECIMALS)
                .checked_mul(U256::from(fee_per_gas))
                .unwrap()
                .into_limbs(),
        ),
        rewards: Rewards {
            local: local_reward.to::<u128>(),
            remote: bridge_hub_rococo_runtime::runtime_types::primitive_types::U256(
                remote_reward.into_limbs(),
            ),
        },
    };

    BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::set_pricing_parameters { params },
    )
}

pub fn force_checkpoint(checkpoint: CheckpointUpdate<512>) -> BridgeHubRuntimeCall {
    BridgeHubRuntimeCall::EthereumBeaconClient(
        snowbridge_pallet_ethereum_client::pallet::Call::force_checkpoint {
            update: Box::new(subxt::utils::Static(checkpoint)),
        },
    )
}
