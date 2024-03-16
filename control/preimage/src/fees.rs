use crate::constants::*;
use alloy_primitives::U256;
use sp_arithmetic::{traits::CheckedDiv, FixedU128};

/// Calculate total fee in native currency to cover all costs of delivering a message to the
/// remote destination.
pub fn calculate_remote_fee(
    exchange_rate: FixedU128,
    fee_per_gas: U256,
    remote_reward: U256,
) -> u128 {
    // Remote fee in ether
    let fee = fee_per_gas
        .saturating_mul(U256::from(TOKEN_TRANSFER_GAS_USED_AT_MOST))
        .saturating_add(remote_reward)
        .to::<u128>();

    // convert to local currency
    let fee = FixedU128::from_inner(fee)
        .checked_div(&exchange_rate)
        .expect("exchange rate is not zero; qed")
        .into_inner();

    // adjust fixed point to match local currency
    let fee = convert_from_ether_decimals(fee);

    fee
}

pub fn convert_from_ether_decimals(value: u128) -> u128 {
    let decimals = ETHER_DECIMALS.saturating_sub(POLKADOT_DECIMALS) as u32;
    let denom = 10u128.saturating_pow(decimals);
    value
        .checked_div(denom)
        .expect("divisor is non-zero; qed")
        .into()
}
