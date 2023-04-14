//! BasicOutboundChannel pallet benchmarking
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::OnInitialize;

#[allow(unused_imports)]
use crate::Pallet as OutboundQueue;

benchmarks! {
	where_clause {
		where
			T::AccountId: AsRef<[u8]>,
	}
	// Benchmark `on_initialize` under worst case conditions, i.e. messages
	// in queue are committed.
	on_commit {
		let m in 1 .. T::MaxMessagesPerCommit::get();
		let p in 0 .. T::MaxMessagePayloadSize::get()-1;

		for _ in 0 .. m {
			let payload: Vec<u8> = (0..).take(p as usize).collect();
			<MessageQueue<T>>::try_append(Message {
				origin: 1000.into(),
				nonce: 0u64,
				handler: 0,
				payload: payload.try_into().unwrap(),
			}).unwrap();
		}

		let block_number = Interval::<T>::get();

	}: { OutboundQueue::<T>::commit(Weight::MAX) }
	verify {
		assert_eq!(<MessageQueue<T>>::get().len(), 0);
	}

	// Benchmark 'on_initialize` for the case where it is a commitment interval
	// but there are no messages in the queue.
	on_commit_no_messages {
		<MessageQueue<T>>::kill();

		let block_number = Interval::<T>::get();

	}: { OutboundQueue::<T>::on_initialize(block_number) }
}

impl_benchmark_test_suite!(OutboundQueue, crate::test::new_tester(), crate::test::Test,);
