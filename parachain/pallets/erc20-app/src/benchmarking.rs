//! ERC20App pallet benchmarking

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use frame_system::RawOrigin;
use sp_core::H160;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use crate::{Address, AssetId, Call, Config as Erc20AppConfig, Pallet as Erc20App};
use snowbridge_core::ChannelId;

use pallet_assets::Config as AssetsConfig;
use snowbridge_basic_channel::outbound::{Config as BasicOutboundChannelConfig, Principal};
use snowbridge_incentivized_channel::outbound::{Config as IncentivizedOutboundChannelConfig, Fee};

use frame_support::traits::{
	fungible::Mutate as FungibleMutate,
	fungibles::{Inspect, Mutate},
};

pub struct Pallet<T: Config>(Erc20App<T>);

pub trait Config:
	AssetsConfig + BasicOutboundChannelConfig + IncentivizedOutboundChannelConfig + Erc20AppConfig
{
}

benchmarks! {
	burn_basic_channel {
		let caller: T::AccountId = whitelisted_caller();
		let token = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let amount: u128 = 500;

		// set principal for basic channel
		Principal::<T>::set(Some(caller.clone()));

		// create wrapped token
		let origin = T::CallOrigin::successful_origin();
		if let Ok(_addr) = T::CallOrigin::try_origin(origin.clone()) {
				<Address<T>>::put(_addr);
		} else {
				return Err("Failed to extract caller address from origin".into());
		}
		let call = Call::<T>::create { token: token };
		call.dispatch_bypass_filter(origin)?;

		let asset_id = <AssetId<T>>::get(token).unwrap();

		T::Assets::mint_into(asset_id, &caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Basic, token, recipient, amount)
	verify {
		assert_eq!(T::Assets::balance(asset_id, &caller), 0);
	}

	burn_incentivized_channel {
		let caller: T::AccountId = whitelisted_caller();
		let token = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let amount = 500;

		// deposit enough money to cover fees
		<T as IncentivizedOutboundChannelConfig>::FeeCurrency::mint_into(&caller, 100)?;
		Fee::<T>::set(50);

		// create wrapped token
		let origin = T::CallOrigin::successful_origin();
		if let Ok(_addr) = T::CallOrigin::try_origin(origin.clone()) {
				<Address<T>>::put(_addr);
		} else {
				return Err("Failed to extract caller address from origin".into());
		}
		let call = Call::<T>::create { token: token };
		call.dispatch_bypass_filter(origin)?;

		let asset_id = <AssetId<T>>::get(token).unwrap();

		T::Assets::mint_into(asset_id, &caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, token, recipient, amount)
	verify {
		assert_eq!(T::Assets::balance(asset_id, &caller), 0);
	}

	// Benchmark `mint` extrinsic under worst case conditions:
	// * `mint` successfully adds amount to recipient account
	mint {
		let origin = T::CallOrigin::successful_origin();
		if let Ok(caller) = T::CallOrigin::try_origin(origin.clone()) {
				<Address<T>>::put(caller);
		} else {
				return Err("Failed to extract caller address from origin".into());
		}

		let token = H160::repeat_byte(2);
		let recipient: T::AccountId = account("recipient", 0, 0);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		let sender = H160::zero();
		let amount = 500;

		// create wrapped token
		let origin = T::CallOrigin::successful_origin();
		if let Ok(_addr) = T::CallOrigin::try_origin(origin.clone()) {
				<Address<T>>::put(_addr);
		} else {
				return Err("Failed to extract caller address from origin".into());
		}
		let call = Call::<T>::create { token: token };
		call.dispatch_bypass_filter(origin.clone())?;

		let asset_id = <AssetId<T>>::get(token).unwrap();

		let call = Call::<T>::mint { token: token, sender: sender, recipient: recipient_lookup, amount : amount, destination: None };

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Assets::balance(asset_id, &recipient), amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_tester(), crate::mock::Test,);
}
