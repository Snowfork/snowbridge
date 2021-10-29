use sp_core::U256;
use sp_runtime::traits::CheckedConversion;

use crate::{BalanceOf, Config};

pub fn unwrap<T: Config>(value: U256, decimals: u32) -> Option<BalanceOf<T>> {
	let granularity = match granularity(decimals) {
		Some(value) => value,
		None => return None,
	};

	let unwrapped = match value.checked_div(granularity) {
		Some(value) => value,
		None => return None,
	};

	unwrapped.low_u128().checked_into()
}

pub fn wrap<T: Config>(value: BalanceOf<T>, decimals: u32) -> Option<U256> {
	let granularity = match granularity(decimals) {
		Some(value) => value,
		None => return None,
	};

	let value_u256 = match value.checked_into::<u128>() {
		Some(value) => U256::from(value),
		None => return None,
	};

	value_u256.checked_mul(granularity)
}

fn granularity(decimals: u32) -> Option<U256> {
	Some(U256::from(u64::checked_pow(10, 18 - decimals)?))
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::mock::{Balance, Test};
	#[test]
	fn should_wrap_without_overflow() {
		// largest possible value
		let max_possible_amount = Balance::MAX;
		let min_possible_decimals = 0;
		assert_ne!(wrap::<Test>(max_possible_amount, min_possible_decimals), None);

		// smallest possible value
		let min_possible_amount = 1;
		let max_possible_decimals = 18;
		assert_ne!(wrap::<Test>(min_possible_amount, max_possible_decimals), None)
	}

	#[test]
	fn should_unwrap_without_overflow() {
		// largest possible value
		let max_possible_amount = U256::from(Balance::MAX);
		let min_possible_decimals = 0;
		assert_ne!(unwrap::<Test>(max_possible_amount, min_possible_decimals), None);

		// smallest possible value
		let min_possible_amount = U256::from(1);
		let max_possible_decimals = 18;
		assert_ne!(unwrap::<Test>(min_possible_amount, max_possible_decimals), None)
	}
}
