//! ETHApp pallet benchmarking
use frame_benchmarking::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use frame_support::traits::fungible::Mutate;

use crate::{Address, Call, Config as EtherAppConfig, Pallet as EtherApp};
use snowbridge_core::ChannelId;

use frame_support::traits::fungible::Inspect;
use pallet_assets::Config as AssetsConfig;
use snowbridge_basic_channel::outbound::{Config as BasicOutboundChannelConfig, Principal};
use snowbridge_incentivized_channel::outbound::{Config as IncentivizedOutboundChannelConfig, Fee};

pub struct Pallet<T: Config>(EtherApp<T>);

pub trait Config:
	AssetsConfig + BasicOutboundChannelConfig + IncentivizedOutboundChannelConfig + EtherAppConfig
{
}

benchmarks! {
	burn_basic_channel {
		let caller: T::AccountId = whitelisted_caller();
		let recipient = H160::repeat_byte(2);
		let amount = 500;

		// set principal for basic channel
		Principal::<T>::set(Some(caller.clone()));

		T::Asset::mint_into(&caller, amount)?;
	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Basic, recipient, amount)
	verify {
		assert_eq!(T::Asset::balance(&caller), 0);
	}

	burn_incentivized_channel {
		let caller: T::AccountId = whitelisted_caller();
		let recipient = H160::repeat_byte(2);
		let amount: u128 = 500;
		let fee: u128 = 50;

		// deposit enough money to cover fees
		Fee::<T>::set(fee);
		T::Asset::mint_into(&caller, fee)?;

		T::Asset::mint_into(&caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
	verify {
		assert_eq!(T::Asset::balance(&caller), 0);
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

		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();
		let amount = 500;

		let call = Call::<T>::mint { sender: sender, recipient: recipient_lookup, amount: amount, destination: None  };
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Asset::balance(&recipient), amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_tester(), crate::mock::Test,);
}
