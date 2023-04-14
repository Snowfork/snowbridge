use super::*;

use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{Everything, GenesisBuild, OnInitialize},
};
use polkadot_parachain::primitives::Id as ParaId;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify},
	MultiSignature,
};
use sp_std::convert::From;

use crate::{self as outbound_channel};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		BasicOutboundChannel: outbound_channel::{Pallet, Config<T>, Storage, Event<T>},
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
	pub const MaxMessagesPerCommit: u32 = 20;
}

impl outbound_channel::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config: outbound_channel::GenesisConfig<Test> =
		outbound_channel::GenesisConfig { interval: 1u64 };
	config.assimilate_storage(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();

	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		BasicOutboundChannel::on_initialize(System::block_number());
	}
}

#[test]
fn test_submit() {
	new_tester().execute_with(|| {
		let parachain_id: &ParaId = &ParaId::new(1000);

		assert_ok!(BasicOutboundChannel::submit(parachain_id, &vec![0, 1, 2]));

		assert_eq!(<Nonce<Test>>::get(parachain_id), 1);
		assert_eq!(<MessageQueue<Test>>::get().len(), 1);
	});
}

#[test]
fn test_submit_exceeds_queue_limit() {
	new_tester().execute_with(|| {
		let parachain_id: &ParaId = &ParaId::new(1000);

		let max_messages = MaxMessagesPerCommit::get();
		(0..max_messages)
			.for_each(|_| BasicOutboundChannel::submit(parachain_id, &vec![0, 1, 2]).unwrap());

		assert_noop!(
			BasicOutboundChannel::submit(parachain_id, &vec![0, 1, 2]),
			Error::<Test>::QueueSizeLimitReached,
		);
	})
}

#[test]
fn test_submit_exceeds_payload_limit() {
	new_tester().execute_with(|| {
		let parachain_id: &ParaId = &ParaId::new(1000);

		let max_payload_bytes = MaxMessagePayloadSize::get() - 1;

		let mut payload: Vec<u8> = (0..).take(max_payload_bytes as usize).collect();
		// Make payload oversize
		payload.push(5);
		payload.push(10);

		assert_noop!(
			BasicOutboundChannel::submit(parachain_id, payload.as_slice()),
			Error::<Test>::PayloadTooLarge,
		);
	})
}

#[test]
fn test_commit_single_user() {
	new_tester().execute_with(|| {
		let parachain_id: &ParaId = &ParaId::new(1000);

		assert_ok!(BasicOutboundChannel::submit(parachain_id, &vec![0, 1, 2]));
		run_to_block(2);
		BasicOutboundChannel::commit(Weight::MAX);

		assert_eq!(<Nonce<Test>>::get(parachain_id), 1);
		assert_eq!(<MessageQueue<Test>>::get().len(), 0);
	})
}

#[test]
fn test_commit_multi_user() {
	new_tester().execute_with(|| {
		let alice: &ParaId = &ParaId::new(1000);
		let bob: &ParaId = &ParaId::new(1000);

		assert_ok!(BasicOutboundChannel::submit(alice, &vec![0, 1, 2]));
		assert_ok!(BasicOutboundChannel::submit(bob, &vec![0, 1, 2]));
		run_to_block(2);
		BasicOutboundChannel::commit(Weight::MAX);

		assert_eq!(<Nonce<Test>>::get(alice), 1);
		assert_eq!(<Nonce<Test>>::get(bob), 1);
		assert_eq!(<MessageQueue<Test>>::get().len(), 0);
	})
}
