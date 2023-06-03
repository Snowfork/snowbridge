use super::*;

use frame_support::{
	assert_err, assert_noop, parameter_types,
	traits::{Everything, Footprint, Hooks, ProcessMessageError},
	weights::WeightMeter,
	BoundedSlice,
};

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify},
	BoundedVec, MultiSignature,
};
use sp_std::convert::From;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		OutboundQueue: crate::{Pallet, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MaxMessagePayloadSize: u32 = 256;
	pub const MaxMessagesPerBlock: u32 = 20;
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type MessageQueue = FakeMessageQueue;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerBlock = MaxMessagesPerBlock;
	type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

parameter_types! {
	pub const MaxMessageLen: u32 = 512;
}

pub struct FakeMessageQueue;

impl EnqueueMessage<crate::AggregateMessageOrigin> for FakeMessageQueue {
	type MaxMessageLen = MaxMessageLen;

	fn enqueue_message(
		message: BoundedSlice<u8, Self::MaxMessageLen>,
		origin: crate::AggregateMessageOrigin,
	) {
		let mut meter = WeightMeter::max_limit();
		let _ = OutboundQueue::process_message(&message, origin, &mut meter);
	}
	fn enqueue_messages<'a>(
		_: impl Iterator<Item = BoundedSlice<'a, u8, Self::MaxMessageLen>>,
		_: crate::AggregateMessageOrigin,
	) {
	}
	fn sweep_queue(_: crate::AggregateMessageOrigin) {}
	fn footprint(_: crate::AggregateMessageOrigin) -> Footprint {
		Footprint::default()
	}
}

#[test]
fn submit_messages_from_multiple_origins_and_commit() {
	new_tester().execute_with(|| {
		for para_id in 1000..1004 {
			let message = OutboundMessage {
				id: H256::repeat_byte(1).into(),
				origin: para_id.into(),
				gateway: [1u8; 32].into(),
				payload: (0..100).map(|_| 1u8).collect::<Vec<u8>>(),
			};

			let result = OutboundQueue::validate(&message);
			assert!(result.is_ok());
			let ticket = result.unwrap();

			OutboundQueue::submit(ticket);
			assert_eq!(<Nonce<Test>>::get(message.origin), 1);
		}

		OutboundQueue::on_finalize(System::block_number());

		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
	});
}

#[test]
fn submit_message_fail_too_large() {
	new_tester().execute_with(|| {
		let message = OutboundMessage {
			id: H256::repeat_byte(1).into(),
			origin: 1000.into(),
			gateway: [1u8; 32].into(),
			payload: (0..1000).map(|_| 1u8).collect::<Vec<u8>>(),
		};

		assert_err!(OutboundQueue::validate(&message), SubmitError::MessageTooLarge);
	});
}

#[test]
fn commit_exits_early_if_no_processed_messages() {
	new_tester().execute_with(|| {
		// on_finalize should do nothing, nor should it panic
		OutboundQueue::on_finalize(System::block_number());

		let digest = System::digest();
		let digest_items = digest.logs();
		assert_eq!(digest_items.len(), 0);
	});
}

#[test]
fn process_message_yields_on_max_messages_per_block() {
	new_tester().execute_with(|| {
		for _ in 0..<Test as Config>::MaxMessagesPerBlock::get() {
			MessageLeaves::<Test>::append(H256::zero())
		}

		let origin = AggregateMessageOrigin::Parachain(1000.into());
		let message = (0..100).map(|_| 1u8).collect::<Vec<u8>>();
		let message: BoundedVec<u8, MaxEnqueuedMessageSizeOf<Test>> = message.try_into().unwrap();

		let mut meter = WeightMeter::max_limit();

		assert_noop!(
			OutboundQueue::process_message(&message.as_bounded_slice(), origin, &mut meter),
			ProcessMessageError::Yield
		);
	})
}

#[test]
fn process_message_fails_on_overweight_message() {
	new_tester().execute_with(|| {
		let origin = AggregateMessageOrigin::Parachain(1000.into());
		let message = (0..100).map(|_| 1u8).collect::<Vec<u8>>();
		let message: BoundedVec<u8, MaxEnqueuedMessageSizeOf<Test>> = message.try_into().unwrap();

		let mut meter = WeightMeter::from_limit(Weight::from_parts(1, 1));

		assert_noop!(
			OutboundQueue::process_message(&message.as_bounded_slice(), origin, &mut meter),
			ProcessMessageError::Overweight(<Test as Config>::WeightInfo::do_process_message())
		);
	})
}
