//! Benchmarking setup for pallet-template
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks(
	where
		T::AccountId: AsRef<[u8]>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn remark() {
		let caller: T::AccountId = whitelisted_caller();
		let data: Vec<u8> = [1u8; 256].into();

		#[extrinsic_call]
		remark(RawOrigin::Signed(caller), data);
	}

	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}
