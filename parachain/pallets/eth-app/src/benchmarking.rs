//! ETHApp pallet benchmarking
use super::*;

use frame_system::RawOrigin;
use frame_support::traits::UnfilteredDispatchable;
use frame_benchmarking::{account, benchmarks, whitelisted_caller, impl_benchmark_test_suite};
use sp_core::H160;

#[allow(unused_imports)]
use crate::Pallet as ETHApp;

benchmarks! {
	// Benchmark `burn` extrinsic under worst case conditions:
	// * `burn` successfully substracts amount from caller account
	// * The channel executes incentivization logic
	burn {
		let caller: T::AccountId = whitelisted_caller();
		let recipient = H160::repeat_byte(2);
		let amount: U256 = 500.into();

		T::Asset::deposit(&caller, amount)?;

	}: _(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
		assert_eq!(T::Asset::balance(&caller), U256::zero());
	}

	// Benchmark `mint` extrinsic under worst case conditions:
	// * `mint` successfully adds amount to recipient account
	mint {
		let origin = T::CallOrigin::successful_origin();
		if let Ok(caller) = T::CallOrigin::try_origin(origin.clone()) {
			<Address<T>>::put(caller);
		} else {
			return Err("Failed to extract caller address from origin");
		}

		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();
		let amount: U256 = 500.into();

		let call = Call::<T>::mint(sender, recipient_lookup, amount);

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Asset::balance(&recipient), amount);
	}
}

impl_benchmark_test_suite!(
	ETHApp,
	crate::mock::new_tester(),
	crate::mock::Test,
);
