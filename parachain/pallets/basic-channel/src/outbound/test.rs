use super::*;

use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{Everything, GenesisBuild, OnInitialize},
};
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify},
	MultiSignature,
};
use sp_std::convert::From;

use crate::outbound as basic_outbound_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		BasicOutboundChannel: basic_outbound_channel::{Pallet, Config<T>, Storage, Event<T>},
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
	pub const MaxMessagePayloadSize: u32 = 128;
	pub const MaxMessagesPerCommit: u32 = 5;
}

impl basic_outbound_channel::Config for Test {
	type SourceId = AccountId;
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config: basic_outbound_channel::GenesisConfig<Test> =
		basic_outbound_channel::GenesisConfig { interval: 1u64 };
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
		let target = H160::zero();
		let who: &AccountId = &Keyring::Bob.into();

		assert_ok!(BasicOutboundChannel::submit(who, target, &vec![0, 1, 2]));

		assert_eq!(<Nonce<Test>>::get(who), 0);
		assert_eq!(<MessageQueue<Test>>::get().len(), 1);
	});
}

#[test]
fn test_submit_exceeds_queue_limit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: &AccountId = &Keyring::Bob.into();

		let max_messages = MaxMessagesPerCommit::get();
		(0..max_messages)
			.for_each(|_| BasicOutboundChannel::submit(who, target, &vec![0, 1, 2]).unwrap());

		assert_noop!(
			BasicOutboundChannel::submit(who, target, &vec![0, 1, 2]),
			Error::<Test>::QueueSizeLimitReached,
		);
	})
}

#[test]
fn test_submit_exceeds_payload_limit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: &AccountId = &Keyring::Bob.into();

		let max_payload_bytes = MaxMessagePayloadSize::get();
		let payload: Vec<u8> = (0..).take(max_payload_bytes as usize + 1).collect();

		assert_noop!(
			BasicOutboundChannel::submit(who, target, payload.as_slice()),
			Error::<Test>::PayloadTooLarge,
		);
	})
}

#[test]
fn test_commit_single_user() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: &AccountId = &Keyring::Bob.into();

		assert_ok!(BasicOutboundChannel::submit(who, target, &vec![0, 1, 2]));
		run_to_block(2);

		assert_eq!(<Nonce<Test>>::get(who), 1);
		assert_eq!(<MessageQueue<Test>>::get().len(), 0);
	})
}

#[test]
fn test_commit_multi_user() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let alice: &AccountId = &Keyring::Alice.into();
		let bob: &AccountId = &Keyring::Bob.into();

		assert_ok!(BasicOutboundChannel::submit(alice, target, &vec![0, 1, 2]));
		assert_ok!(BasicOutboundChannel::submit(bob, target, &vec![0, 1, 2]));
		run_to_block(2);

		assert_eq!(<Nonce<Test>>::get(alice), 1);
		assert_eq!(<Nonce<Test>>::get(bob), 1);
		assert_eq!(<MessageQueue<Test>>::get().len(), 0);
	})
}
