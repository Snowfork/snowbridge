//! ERC20App pallet benchmarking

use frame_benchmarking::{account, benchmarks, whitelisted_caller, BenchmarkError};
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};
use frame_system::RawOrigin;
use sp_core::{H160, U256};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

use crate::{Address, Call, Config as Erc20AppConfig, Pallet as Erc20App};
use snowbridge_core::ChannelId;

use snowbridge_assets::Config as AssetsConfig;
use snowbridge_basic_channel::outbound::{Config as BasicOutboundChannelConfig, Principal};
use snowbridge_core::{AssetId, MultiAsset, SingleAsset};
use snowbridge_incentivized_channel::outbound::{Config as IncentivizedOutboundChannelConfig, Fee};
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
		let amount: U256 = 500.into();

		// set principal for basic channel
		Principal::<T>::set(caller.clone());

		T::Assets::deposit(AssetId::Token(token), &caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Basic, token, recipient, amount)
	verify {
		assert_eq!(T::Assets::balance(AssetId::Token(token), &caller), U256::zero());
	}

	burn_incentivized_channel {
		let caller: T::AccountId = whitelisted_caller();
		let token = H160::repeat_byte(1);
		let recipient = H160::repeat_byte(2);
		let amount: U256 = 500.into();

		// deposit enough money to cover fees
		<T as IncentivizedOutboundChannelConfig>::FeeCurrency::deposit(&caller, 100.into())?;
		Fee::<T>::set(50.into());

		T::Assets::deposit(AssetId::Token(token), &caller, amount)?;

	}: burn(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, token, recipient, amount)
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

		let call = Call::<T>::mint { token: token, sender: sender, recipient: recipient_lookup, amount : amount, para_id: None};

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Assets::balance(AssetId::Token(token), &recipient), amount);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_tester(), crate::mock::Test,);
}
