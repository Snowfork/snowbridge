// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use codec::Encode;
use frame_benchmarking::v2::*;
use snowbridge_core::outbound::Command;

#[allow(unused_imports)]
use crate::Pallet as OutboundQueue;

#[benchmarks(
	where
		<T as Config>::MaxMessagePayloadSize: Get<u32>,
)]
mod benchmarks {
	use super::*;
	use snowbridge_core::outbound::Priority;

	/// Benchmark for processing a message payload of length `x`.
	#[benchmark]
	fn do_process_message() -> Result<(), BenchmarkError> {
		let enqueued_message = EnqueuedMessage {
			id: H256::zero().into(),
			origin: 1000.into(),
			command: Command::CreateAgent { agent_id: H256::zero() },
		};
		let encoded_enqueued_message = enqueued_message.encode();

		#[block]
		{
			let _ =
				OutboundQueue::<T>::do_process_message(&encoded_enqueued_message, Priority::Normal);
		}

		assert_eq!(MessageLeaves::<T>::decode_len().unwrap(), 1);

		Ok(())
	}

	/// Benchmark for producing final messages commitment
	#[benchmark]
	fn on_finalize() -> Result<(), BenchmarkError> {
		// Assume worst case, where `MaxMessagesPerBlock` messages need to be committed.
		for i in 0..T::MaxMessagesPerBlock::get() {
			let leaf_data: [u8; 1] = [i as u8];
			let leaf = <T as Config>::Hashing::hash(&leaf_data);
			MessageLeaves::<T>::append(leaf);
		}

		#[block]
		{
			OutboundQueue::<T>::commit_messages();
		}

		Ok(())
	}

	impl_benchmark_test_suite!(OutboundQueue, crate::test::new_tester(), crate::test::Test,);
}
