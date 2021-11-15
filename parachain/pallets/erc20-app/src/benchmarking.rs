//! ERC20App pallet benchmarking
use super::*;

use frame_benchmarking::{
	account, benchmarks, impl_benchmark_test_suite, whitelisted_caller, BenchmarkError,
};
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_core::H160;

#[allow(unused_imports)]
use crate::Pallet as ERC20App;

benchmarks! {
	// Benchmark `burn` extrinsic under worst case conditions:
	// * `burn` successfully substracts amount from caller account
	// * The channel executes incentivization logic
	burn {
		let caller: T::AccountId = whitelisted_caller();
		let token = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let amount: U256 = 500.into();

		T::Assets::deposit(AssetId::Token(token), &caller, amount)?;

	}: _(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, token, recipient, amount)
	verify {
		assert_eq!(T::Assets::balance(AssetId::Token(token), &caller), U256::zero());
	}

	// Benchmark `mint` extrinsic under worst case conditions:
	// * `mint` successfully adds amount to recipient account
	mint {
		let origin = T::CallOrigin::successful_origin();
		if let Ok(caller) = T::CallOrigin::try_origin(origin.clone()) {
			<Address<T>>::put(caller);
		} else {
			return Err(BenchmarkError::Stop("Failed to extract caller address from origin"));
		}

		let token = H160::repeat_byte(1);
		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();
		let amount: U256 = 500.into();

		let call = Call::<T>::mint { token: token, sender: sender, recipient: recipient_lookup, amount : amount};

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Assets::balance(AssetId::Token(token), &recipient), amount);
	}
}

impl_benchmark_test_suite!(ERC20App, crate::mock::new_tester(), crate::mock::Test,);
