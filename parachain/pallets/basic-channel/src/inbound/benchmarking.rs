//! BasicInboundChannel pallet benchmarking
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::pallet_prelude::Weight;
use frame_system::RawOrigin;
use snowbridge_core::Proof;

use hex_literal::hex;

#[allow(unused_imports)]
use crate::inbound::Pallet as BasicInboundChannel;

#[cfg(test)]
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["86d9ac0bab011917f57b9e9607833b4340f9d4f8"];

const MESSAGE_DATA_0: [u8; 284] = hex!(
	"
	f901199486d9ac0bab011917f57b9e9607833b4340f9d4f8e1a0a5f39ee370f7
	0ab738604498959665b56d7cfdcfc84c052d752404fbc09ae3dfb8e000000000
	000000000000000089b4ab1ef20763630df9743acf155865600daff200000000
	000000000000000004e00e6d2e9ea1e2af553de02a5172120bfa5c3e00000000
	0000000000000000000000000000000000000000000000000000000100000000
	0000000000000000000000000000000000000000000000000000002a00000000
	000000000000000000000000000000000000000000000000000000a000000000
	0000000000000000000000000000000000000000000000000000002061726269
	74726172792d7061796c6f6164000000000000000000000000000000
"
);

benchmarks! {
	where_clause {
		where
			T::AccountId: AsRef<[u8]>,
	}

	submit {
		let origin = RawOrigin::Signed(whitelisted_caller::<T::AccountId>());

		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
			dispatch_weight: Weight::from_ref_time(123)
		};

	}: _(origin, message)
	verify {
	}
}

impl_benchmark_test_suite!(
	BasicInboundChannel,
	crate::inbound::test::new_tester(crate::inbound::benchmarking::SOURCE_CHANNEL_ADDR.into()),
	crate::inbound::test::Test,
);
