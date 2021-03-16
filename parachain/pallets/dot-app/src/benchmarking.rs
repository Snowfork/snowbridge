//! DotApp pallet benchmarking

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_system::{RawOrigin};
use frame_support::traits::UnfilteredDispatchable;
use frame_benchmarking::{account, benchmarks, whitelisted_caller, impl_benchmark_test_suite};
use hex_literal::hex;
use sp_core::H160;
use sp_runtime::traits::Zero;

use crate::Module as DotApp;

const ETH_CONTRACT_ADDRESS: [u8; 20] = hex!["b1185ede04202fe62d38f5db72f71e38ff3e8305"];

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

	}: lock(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
        assert!(!balance.is_zero() && !amount.is_zero());
        assert_eq!(T::Currency::free_balance(&caller), balance - amount);
		assert_eq!(T::Currency::free_balance(&lock_account), amount);
	}

    // Benchmark `unlock` extrinsic under worst case conditions:
    // * The amount is successfully unlocked
	unlock {
        let caller: H160 = ETH_CONTRACT_ADDRESS.into();
        Address::put(caller);

        let existential_deposit = T::Currency::minimum_balance();
        let lock_account = DotApp::<T>::account_id();
        let recipient: T::AccountId = account("recipient", 0, 0);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
        let sender = H160::zero();

        let balance = existential_deposit * 10u32.into();
        // The balance remaining after unlock will be < existential deposit.
        // This shouldn't trigger account reaping because we're using
        // the 'KeepAlive' flag. But just in case that ever changes...
        let amount = existential_deposit * 9u32.into() + 1u32.into();
        let amount_wrapped = wrap::<T>(amount, T::Decimals::get()).unwrap();

        T::Currency::make_free_balance_be(&lock_account, balance);

        let call = Call::<T>::unlock(sender, recipient_lookup, amount_wrapped);
        let origin = T::CallOrigin::successful_origin();

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
        assert!(!balance.is_zero() && !amount.is_zero());
        assert_eq!(T::Currency::free_balance(&lock_account), balance - amount);
		assert_eq!(T::Currency::free_balance(&recipient), amount);
	}
}

impl_benchmark_test_suite!(
	DotApp,
	crate::mock::new_tester(),
	crate::mock::Test,
);
