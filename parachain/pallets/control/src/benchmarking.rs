// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as SnowbridgeControl;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use snowbridge_core::outbound::OperatingMode;
use sp_core::{Get, H160, H256};
use sp_std::vec;
use sp_std::vec::Vec;
use frame_support::traits::EnsureOrigin;

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
		let origin = T::AgentOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin);

		Ok(())
	}

	#[benchmark]
	fn create_channel() -> Result<(), BenchmarkError> {
		let origin = T::AgentOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		frame_support::assert_ok!(SnowbridgeControl::<T>::create_agent(origin.clone() as T::RuntimeOrigin));

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin);

		Ok(())
	}

	#[benchmark]
	fn update_channel() -> Result<(), BenchmarkError> {
		let origin = T::AgentOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;

		frame_support::assert_ok!(SnowbridgeControl::<T>::create_agent(
			origin.clone() as T::RuntimeOrigin
		));
		frame_support::assert_ok!(SnowbridgeControl::<T>::create_channel(
			origin.clone() as T::RuntimeOrigin
		));

		#[extrinsic_call]
		_(origin as T::RuntimeOrigin, OperatingMode::RejectingOutboundMessages, 1, 1);

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
