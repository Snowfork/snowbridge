// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as SnowbridgeControl;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use snowbridge_core::outbound::OperatingMode;
use sp_core::Get;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn upgrade(x: Linear<0, { T::MaxUpgradeDataSize::get() - 1 }>) -> Result<(), BenchmarkError> {
		let impl_address = H160::repeat_byte(1);
		let impl_code_hash = H256::repeat_byte(1);
		let params: Vec<u8> = (0..x).map(|_| 1u8).collect();

		#[extrinsic_call]
		_(RawOrigin::Root, impl_address, impl_code_hash, Some(params));

		Ok(())
	}

	#[benchmark]
	fn create_agent() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller));

		Ok(())
	}

	#[benchmark]
	fn create_channel() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		frame_support::assert_ok!(SnowbridgeControl::<T>::create_agent(
			RawOrigin::Signed(caller.clone()).into()
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller));

		Ok(())
	}

	#[benchmark]
	fn update_channel() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		frame_support::assert_ok!(SnowbridgeControl::<T>::create_agent(
			RawOrigin::Signed(caller.clone()).into()
		));
		frame_support::assert_ok!(SnowbridgeControl::<T>::create_channel(
			RawOrigin::Signed(caller.clone()).into()
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), OperatingMode::RejectingOutboundMessages, 1, 1);

		Ok(())
	}

	#[benchmark]
	fn set_operating_mode() -> Result<(), BenchmarkError> {
		#[extrinsic_call]
		_(RawOrigin::Root, OperatingMode::RejectingOutboundMessages);

		Ok(())
	}

	#[benchmark]
	fn transfer_native_from_agent() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		frame_support::assert_ok!(SnowbridgeControl::<T>::create_agent(
			RawOrigin::Signed(caller.clone()).into()
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), H160::default(), 1);

		Ok(())
	}

	impl_benchmark_test_suite!(SnowbridgeControl, crate::mock::new_test_ext(), crate::mock::Test);
}
