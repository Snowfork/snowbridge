//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn upgrade() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let upgrade_task = H160::repeat_byte(3);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), upgrade_task);

		Ok(())
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
