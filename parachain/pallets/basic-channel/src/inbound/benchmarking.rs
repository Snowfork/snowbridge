//! BasicInboundChannel pallet benchmarking
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
// use frame_support::traits::OriginTrait;
// use frame_system::pallet_prelude::OriginFor;
// use sp_keyring::AccountKeyring as Keyring;
// use sp_runtime::{
// 	traits::{IdentifyAccount, Verify},
// 	MultiSignature,
// };
// use sp_std::convert::From;
// use snowbridge_core::Proof;

#[allow(unused_imports)]
use crate::inbound::Pallet as BasicInboundChannel;

use crate::inbound::test::new_tester;

use hex_literal::hex;

// pub type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;

const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["86d9ac0bab011917f57b9e9607833b4340f9d4f8"];
// const MESSAGE_DATA_0: [u8; 284] = hex!(
// 	"
// 	f901199486d9ac0bab011917f57b9e9607833b4340f9d4f8e1a0a5f39ee370f7
// 	0ab738604498959665b56d7cfdcfc84c052d752404fbc09ae3dfb8e000000000
// 	000000000000000089b4ab1ef20763630df9743acf155865600daff200000000
// 	000000000000000004e00e6d2e9ea1e2af553de02a5172120bfa5c3e00000000
// 	0000000000000000000000000000000000000000000000000000000100000000
// 	0000000000000000000000000000000000000000000000000000002a00000000
// 	000000000000000000000000000000000000000000000000000000a000000000
// 	0000000000000000000000000000000000000000000000000000002061726269
// 	74726172792d7061796c6f6164000000000000000000000000000000
// "
// );

fn new_tester_benchmarking() -> sp_io::TestExternalities {
	new_tester(snowbridge_ethereum::H160(SOURCE_CHANNEL_ADDR))
}

benchmarks! {
	where_clause {
		where
			T::AccountId: AsRef<[u8]>,
	}

	// Benchmark `submit` under worst case conditions, i.e. largest
	// possible message.
	submit {
		// let relayer: AccountId = Keyring::Bob.into();
		// let origin = OriginFor::<T>::signed(relayer);

		// let message = Message {
		// 	data: MESSAGE_DATA_0.into(),
		// 	proof: Proof {
		// 		block_hash: Default::default(),
		// 		tx_index: Default::default(),
		// 		data: Default::default(),
		// 	},
		// 	dispatch_weight: 123
		// };
	}: {}
	verify {
	}

	// // Benchmark `submit` for the best case, i.e. smallest possible message
	// submit_smallest_message {
	// 	<MessageQueue<T>>::try_append(EnqueuedMessage {
	// 		account: account("", 0, 0),
	// 		message: Message {
	// 			id: 0u64,
	// 			target: H160::zero(),
	// 			payload: vec![1u8; T::MaxMessagePayloadSize::get() as usize].try_into().unwrap(),
	// 		}
	// 	}).unwrap();

	// 	Interval::<T>::put::<T::BlockNumber>(10u32.into());
	// 	let block_number: T::BlockNumber = 11u32.into();

	// }: { BasicInboundChannel::<T>::on_initialize(block_number) }
	// verify {
	// 	assert_eq!(<MessageQueue<T>>::get().len(), 1);
	// }
}

impl_benchmark_test_suite!(
	BasicInboundChannel,
	super::new_tester_benchmarking(),
	crate::inbound::test::Test,
);
