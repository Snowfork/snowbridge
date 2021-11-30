//! IncentivizedInboundChannel pallet benchmarking

use super::*;

use frame_benchmarking::{benchmarks, BenchmarkError};

#[allow(unused_imports)]
use crate::inbound::Pallet as IncentivizedInboundChannel;

// This collection of benchmarks should include a benchmark for each
// call dispatched by the channel, i.e. each "app" pallet function
// that can be invoked by MessageDispatch. The most expensive call
// should be used in the `submit` benchmark.
//
// We rely on configuration via chain spec of the app pallets because
// we don't have access to their storage here.
benchmarks! {
	// Benchmark `set_reward_fraction` under worst case conditions:
	// * The origin is authorized, i.e. equals UpdateOrigin
	set_reward_fraction {
		let authorized_origin = match T::UpdateOrigin::successful_origin().into() {
			Ok(raw) => raw,
			Err(_) => return Err(BenchmarkError::Stop("Failed to get raw origin from origin")),
		};

		// Pick a value that is different from the initial RewardFraction
		let fraction = Perbill::from_percent(50);
		assert!(<RewardFraction<T>>::get() != fraction);

	}: _(authorized_origin, fraction)
	verify {
		assert_eq!(<RewardFraction<T>>::get(), fraction);
	}

	impl_benchmark_test_suite!(
		IncentivizedInboundChannel,
		crate::inbound::test::new_tester(Default::default()),
		crate::inbound::test::Test,
	);
}
