//! ERC20App pallet benchmarking
use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_core::H160;

use frame_support::traits::fungibles::{Inspect, Mutate};

#[allow(unused_imports)]
use crate::Pallet as ERC20App;

benchmarks! {
	// Benchmark `burn` extrinsic under worst case conditions:
	// * `burn` successfully substracts amount from caller account
	// * The channel executes incentivization logic
	burn {
		let caller: T::AccountId = whitelisted_caller();
		let token = H160::repeat_byte(2);
		let recipient = H160::repeat_byte(3);
		let amount = 500;

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

	}: _(RawOrigin::Signed(caller.clone()), ChannelId::Incentivized, token, recipient, amount)
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

		let call = Call::<T>::mint { token, sender, recipient: recipient_lookup, amount };

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		assert_eq!(T::Assets::balance(asset_id, &recipient), amount);
	}

	impl_benchmark_test_suite!(ERC20App, crate::mock::new_tester(), crate::mock::Test,);
}
