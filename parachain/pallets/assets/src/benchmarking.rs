//! Assets pallet benchmarking

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_core::H160;

#[allow(unused_imports)]
use crate::Pallet as Assets;

fn set_balance<T: Config>(asset_id: &AssetId, who: &T::AccountId, amount: &U256) {
	TotalIssuance::insert(asset_id, amount);
	Balances::<T>::insert(asset_id, who, amount);
}

fn get_balance<T: Config>(asset_id: &AssetId, who: &T::AccountId) -> U256 {
	Balances::<T>::get(asset_id, who)
}

benchmarks! {
	// Benchmark `transfer` extrinsic under worst case conditions, i.e. successful transfer:
	// * `transfer` will substract amount from caller account
	// * `transfer` will add amount to destination account
	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let initial_amount = U256::from_str_radix("1000000000000000000", 10).unwrap();
		let transfer_amount = U256::from_str_radix("500000000000000000", 10).unwrap();
		let token = AssetId::Token(H160::zero());
		let dest: T::AccountId = account("recipient", 0, 0);
		let dest_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(dest.clone());

		set_balance::<T>(&token, &caller, &initial_amount);

	}: _(RawOrigin::Signed(caller.clone()), token, dest_lookup, transfer_amount)
	verify {
		assert_eq!(get_balance::<T>(&token, &caller), initial_amount - transfer_amount);
		assert_eq!(get_balance::<T>(&token, &dest), transfer_amount);
	}
}

impl_benchmark_test_suite!(Assets, crate::mock::new_tester(), crate::mock::Test,);
