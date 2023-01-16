//! BasicOutboundChannel pallet benchmarking
use super::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::OnInitialize;

#[allow(unused_imports)]
use crate::outbound::Pallet as BasicOutboundChannel;

benchmarks! {
	where_clause {
		where
			T::AccountId: AsRef<[u8]>,
	}
	// Benchmark `on_initialize` under worst case conditions, i.e. messages
	// in queue are committed.
	on_initialize {
		let m in 1 .. T::MaxMessagesPerCommit::get();
		let p in 0 .. T::MaxMessagePayloadSize::get();

		for _ in 0 .. m {
			let payload: Vec<u8> = (0..).take(p as usize).collect();
			<MessageQueue<T>>::try_append(EnqueuedMessage {
				account: account("", 0, 0),
				message: Message {
					id: 0u64,
					target: H160::zero(),
					payload: payload.try_into().unwrap(),
				}
			}).unwrap();
		}

		let block_number = Interval::<T>::get();

	}: { BasicOutboundChannel::<T>::commit(Weight::MAX) }
	verify {
		assert_eq!(<MessageQueue<T>>::get().len(), 0);
	}

	// Benchmark 'on_initialize` for the best case, i.e. nothing is done
	// because it's not a commitment interval.
	on_initialize_non_interval {
		<MessageQueue<T>>::try_append(EnqueuedMessage {
			account: account("", 0, 0),
			message: Message {
				id: 0u64,
				target: H160::zero(),
				payload: vec![1u8; T::MaxMessagePayloadSize::get() as usize].try_into().unwrap(),
			}
		}).unwrap();

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
}

impl_benchmark_test_suite!(
	BasicOutboundChannel,
	crate::outbound::test::new_tester(),
	crate::outbound::test::Test,
);
