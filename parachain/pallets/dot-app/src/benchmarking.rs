//! DotApp pallet benchmarking

use frame_benchmarking::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_support::traits::{
	tokens::currency::Currency, EnsureOrigin, Get, UnfilteredDispatchable,
};

use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::traits::{StaticLookup, Zero};
use sp_std::prelude::*;

use crate::{primitives::wrap, Address, Call, Config as DotAppConfig, Pallet as DotApp};
use snowbridge_core::ChannelId;

use snowbridge_assets::Config as AssetsConfig;
use snowbridge_basic_channel::outbound::{Config as BasicOutboundChannelConfig, Principal};
use snowbridge_core::SingleAsset;
use snowbridge_incentivized_channel::outbound::{Config as IncentivizedOutboundChannelConfig, Fee};

pub struct Pallet<T: Config>(DotApp<T>);

pub trait Config:
	AssetsConfig + BasicOutboundChannelConfig + IncentivizedOutboundChannelConfig + DotAppConfig
{
}

benchmarks! {
	lock_basic_channel {
		let existential_deposit = T::Currency::minimum_balance();
		let caller: T::AccountId = whitelisted_caller();
		let lock_account = DotApp::<T>::account_id();
		let recipient = H160::zero();

		// set principal for basic channel
		Principal::<T>::set(caller.clone());

		let balance = existential_deposit * 10u32.into();
		// The amount is chosen such that balance - amount < existential_deposit
		// so that the account is reaped
		let amount = existential_deposit * 9u32.into() + 1u32.into();

		// Create DOT account for caller
		T::Currency::make_free_balance_be(&caller, balance);

		// Create account to store locked DOT
		T::Currency::make_free_balance_be(&lock_account, 0u32.into());

	}: lock(RawOrigin::Signed(caller.clone()), ChannelId::Basic, recipient, amount)
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&caller), Zero::zero());
		assert_eq!(T::Currency::free_balance(&lock_account), amount);
	}

	lock_incentivized_channel {
		let existential_deposit = T::Currency::minimum_balance();
		let caller: T::AccountId = whitelisted_caller();
		let lock_account = DotApp::<T>::account_id();
		let recipient = H160::zero();

		// deposit enough money to cover fees
		<T as IncentivizedOutboundChannelConfig>::FeeCurrency::deposit(&caller, 100.into())?;
		Fee::<T>::set(50.into());

		let balance = existential_deposit * 10u32.into();
		// The amount is chosen such that balance - amount < existential_deposit
		// so that the account is reaped
		let amount = existential_deposit * 9u32.into() + 1u32.into();

		// Create DOT account for caller
		T::Currency::make_free_balance_be(&caller, balance);

		// Create account to store locked DOT
		T::Currency::make_free_balance_be(&lock_account, 0u32.into());

	}: lock(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&caller), Zero::zero());
		assert_eq!(T::Currency::free_balance(&lock_account), amount);
	}

	// Benchmark `unlock` extrinsic under worst case conditions:
	// * The amount is successfully unlocked
	unlock {
		let origin = T::CallOrigin::successful_origin();
		if let Ok(caller) = T::CallOrigin::try_origin(origin.clone()) {
			<Address<T>>::put(caller);
		} else {
			return Err(BenchmarkError::Stop("Failed to extract caller address from origin"));
		}

		let existential_deposit = T::Currency::minimum_balance();
		let lock_account = DotApp::<T>::account_id();
		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();

		let balance = existential_deposit * 10u32.into();
		let amount = existential_deposit * 8u32.into();
		let amount_wrapped = wrap::<T>(amount, T::Decimals::get()).unwrap();

		// Create DOT account for caller
		T::Currency::make_free_balance_be(&recipient, 0u32.into());

		// Create account to store locked DOT
		T::Currency::make_free_balance_be(&lock_account, balance);

		let call = Call::<T>::unlock { sender: sender, recipient: recipient_lookup, amount: amount_wrapped };

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert!(!balance.is_zero() && !amount.is_zero());
		assert_eq!(T::Currency::free_balance(&lock_account), balance - amount);
		assert_eq!(T::Currency::free_balance(&recipient), amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_tester(), crate::mock::Test);
}
