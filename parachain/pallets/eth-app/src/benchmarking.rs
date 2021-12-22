//! ETHApp pallet benchmarking
use frame_benchmarking::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use frame_system::RawOrigin;
use sp_core::{H160, U256};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use crate::{Address, Call, Config as EtherAppConfig, Pallet as EtherApp};
use snowbridge_core::ChannelId;

use snowbridge_assets::Config as AssetsConfig;
use snowbridge_basic_channel::outbound::{Config as BasicOutboundChannelConfig, Principal};
use snowbridge_core::SingleAsset;
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
		let amount: U256 = 500.into();

		// set principal for basic channel
		Principal::<T>::set(caller.clone());

		T::Asset::deposit(&caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Basic, recipient, amount)
	verify {
		assert_eq!(T::Asset::balance(&caller), U256::zero());
	}

	burn_incentivized_channel {
		let caller: T::AccountId = whitelisted_caller();
		let recipient = H160::repeat_byte(2);
		let amount: U256 = 500.into();
		let fee: U256 = 50.into();

		// deposit enough money to cover fees
		Fee::<T>::set(fee);
		T::Asset::deposit(&caller, fee)?;

		T::Asset::deposit(&caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, recipient, amount)
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
			return Err(BenchmarkError::Stop("Failed to extract caller address from origin"));
		}

		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();
		let amount: U256 = 500.into();

		let call = Call::<T>::mint { sender: sender, recipient: recipient_lookup, amount: amount, para_id: None };
	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Asset::balance(&recipient), amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_tester(), crate::mock::Test,);
}
