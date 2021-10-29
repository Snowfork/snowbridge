//! DotApp pallet benchmarking
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::traits::Zero;

use crate::Pallet as DotApp;

benchmarks! {
	// Benchmark `lock` extrinsic under worst case conditions:
	// * The amount is successfully locked
	// * The sender account is killed
	// * The channel executes incentivization logic
	lock {
		let existential_deposit = T::Currency::minimum_balance();
		let caller: T::AccountId = whitelisted_caller();
		let lock_account = DotApp::<T>::account_id();
		let recipient = H160::zero();

		let balance = existential_deposit * 10u32.into();
		// The amount is chosen such that balance - amount < existential_deposit
		// so that the account is reaped
		let amount = existential_deposit * 9u32.into() + 1u32.into();

		T::Currency::make_free_balance_be(&caller, balance);
		T::Currency::make_free_balance_be(&lock_account, 0u32.into());

	}: _(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&caller), Zero::zero());
		assert_eq!(T::Currency::free_balance(&lock_account), amount);
	}

	// Benchmark `lock` extrinsic for the average case:
	// * The amount is successfully locked
	// * The sender remains alive
	// * The channel executes incentivization logic
	#[extra]
	lock_sender_alive {
		let existential_deposit = T::Currency::minimum_balance();
		let caller: T::AccountId = whitelisted_caller();
		let lock_account = DotApp::<T>::account_id();
		let recipient = H160::zero();

		let balance = existential_deposit * 10u32.into();
		let amount = existential_deposit * 8u32.into();

		T::Currency::make_free_balance_be(&caller, balance);
		T::Currency::make_free_balance_be(&lock_account, 0u32.into());

	}: lock(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&caller), balance - amount);
		assert_eq!(T::Currency::free_balance(&lock_account), amount);
	}

	// Benchmark `unlock` extrinsic under worst case conditions:
	// * The amount is successfully unlocked
	unlock {
		let origin = T::CallOrigin::successful_origin();
		if let Ok(caller) = T::CallOrigin::try_origin(origin.clone()) {
			<Address<T>>::put(caller);
		} else {
			return Err("Failed to extract caller address from origin");
		}

		let existential_deposit = T::Currency::minimum_balance();
		let lock_account = DotApp::<T>::account_id();
		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();

		let balance = existential_deposit * 10u32.into();
		let amount = existential_deposit * 8u32.into();
		let amount_wrapped = wrap::<T>(amount, T::Decimals::get()).unwrap();

		T::Currency::make_free_balance_be(&lock_account, balance);

		let call = Call::<T>::unlock(sender, recipient_lookup, amount_wrapped);

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&lock_account), balance - amount);
		assert_eq!(T::Currency::free_balance(&recipient), amount);
	}
}

impl_benchmark_test_suite!(DotApp, crate::mock::new_tester(), crate::mock::Test,);
