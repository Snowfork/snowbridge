// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::Get;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn upgrade(x: Linear<0, { T::MaxUpgradeDataSize::get() - 1 }>) -> Result<(), BenchmarkError> {
		let logic = H160::repeat_byte(1);
		let data: Vec<u8> = (0..x).map(|_| 1u8).collect();

		#[extrinsic_call]
		_(RawOrigin::Root, logic, Some(data));

		Ok(())
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
