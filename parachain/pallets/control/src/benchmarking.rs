// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as SnowbridgeControl;
use frame_support::pallet_prelude::EnsureOrigin;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use snowbridge_core::outbound::OperatingMode;


type RuntimeOriginOf<T> = <T as frame_system::Config>::RuntimeOrigin;


fn fund_sovereign_account<T: Config>(sender: &RuntimeOriginOf<T>) -> Result<(), BenchmarkError> {
	let location: MultiLocation = T::AgentOwnerOrigin::ensure_origin(sender.clone()).map_err(|_| BenchmarkError::Weightless)?;
	let sovereign_account = T::SovereignAccountOf::convert_location(&location).ok_or(BenchmarkError::Weightless)?;
	T::Token::mint_into(&sovereign_account, u32::MAX.into()).map_err(|_| BenchmarkError::Weightless)?;
	Ok(())
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn upgrade() -> Result<(), BenchmarkError> {
		let impl_address = H160::repeat_byte(1);
		let impl_code_hash = H256::repeat_byte(1);

		// Assume 256 bytes passed to initializer
		let params: Vec<u8> = (0..256).map(|_| 1u8).collect();

		#[extrinsic_call]
		_(RawOrigin::Root, impl_address, impl_code_hash, Some(Initializer { params, maximum_required_gas: 100000}));

		Ok(())
	}

	#[benchmark]
	fn create_agent() -> Result<(), BenchmarkError> {
		let sender = T::AgentOwnerOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		fund_sovereign_account::<T>(&sender)?;

		#[extrinsic_call]
		_(sender as T::RuntimeOrigin);

		Ok(())
	}

	#[benchmark]
	fn create_channel() -> Result<(), BenchmarkError> {
		let sender = T::ChannelOwnerOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		fund_sovereign_account::<T>(&sender)?;

		SnowbridgeControl::<T>::create_agent(sender.clone()).map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(sender as T::RuntimeOrigin);

		Ok(())
	}

	#[benchmark]
	fn update_channel() -> Result<(), BenchmarkError> {
		let sender = T::ChannelOwnerOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		fund_sovereign_account::<T>(&sender)?;
		SnowbridgeControl::<T>::create_agent(sender.clone()).map_err(|_| BenchmarkError::Weightless)?;
		SnowbridgeControl::<T>::create_channel(sender.clone()).map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(sender as T::RuntimeOrigin, OperatingMode::RejectingOutboundMessages, 1);

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
		let sender = T::AgentOwnerOrigin::try_successful_origin().map_err(|_| BenchmarkError::Weightless)?;
		fund_sovereign_account::<T>(&sender)?;
		SnowbridgeControl::<T>::create_agent(sender.clone()).map_err(|_| BenchmarkError::Weightless)?;

		#[extrinsic_call]
		_(sender as T::RuntimeOrigin, H160::default(), 1);

		Ok(())
	}

	impl_benchmark_test_suite!(SnowbridgeControl, crate::mock::new_test_ext(), crate::mock::Test);
}
