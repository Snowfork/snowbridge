//! BasicOutboundChannel pallet benchmarking
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, BenchmarkError};
use frame_support::traits::OnInitialize;

#[allow(unused_imports)]
use crate::outbound::Pallet as BasicOutboundChannel;

const SEED: u32 = 0;

benchmarks! {
	// Benchmark `on_initialize` under worst case conditions, i.e. messages
	// in queue are committed.
	on_initialize {
		let m in 1 .. T::MaxMessagesPerCommit::get() as u32;
		let p in 0 .. T::MaxMessagePayloadSize::get() as u32;

		for _ in 0 .. m {
			let payload: Vec<u8> = (0..).take(p as usize).collect();
			<MessageQueue<T>>::append(Message {
				target: H160::zero(),
				nonce: 0u64,
				payload,
			});
		}

		let block_number = Interval::<T>::get();

	}: { BasicOutboundChannel::<T>::on_initialize(block_number) }
	verify {
		assert_eq!(<MessageQueue<T>>::get().len(), 0);
	}

	// Benchmark 'on_initialize` for the best case, i.e. nothing is done
	// because it's not a commitment interval.
	on_initialize_non_interval {
		<MessageQueue<T>>::append(Message {
			target: H160::zero(),
			nonce: 0u64,
			payload: vec![1u8; T::MaxMessagePayloadSize::get() as usize],
		});

		Interval::<T>::put::<T::BlockNumber>(10u32.into());
		let block_number: T::BlockNumber = 11u32.into();

	}: { BasicOutboundChannel::<T>::on_initialize(block_number) }
	verify {
		assert_eq!(<MessageQueue<T>>::get().len(), 1);
	}

	// Benchmark 'on_initialize` for the case where it is a commitment interval
	// but there are no messages in the queue.
	on_initialize_no_messages {
		<MessageQueue<T>>::kill();

		let block_number = Interval::<T>::get();

	}: { BasicOutboundChannel::<T>::on_initialize(block_number) }

	set_principal {
		let authorized_origin = match T::SetPrincipalOrigin::successful_origin().into() {
			Ok(raw) => raw,
			Err(_) => return Err(BenchmarkError::Stop("Failed to get raw origin from origin")),
		};
		let alice = T::Lookup::unlookup(account("alice", 0, SEED));
	}: _(authorized_origin, alice)
	verify {
		assert_eq!(<Principal<T>>::get(), account("alice", 0, SEED));
	}
}

impl_benchmark_test_suite!(
	BasicOutboundChannel,
	crate::outbound::test::new_tester(),
	crate::outbound::test::Test,
);
