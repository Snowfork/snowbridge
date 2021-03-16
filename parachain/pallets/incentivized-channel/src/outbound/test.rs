use super::*;

use frame_support::{
	assert_ok,
	parameter_types,
};
use sp_core::{H160, H256};
use sp_io::TestExternalities;
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	DigestItem,
	MultiSignature,
	traits::{BlakeTwo256, Keccak256, IdentityLookup, IdentifyAccount, Verify},
	testing::Header,
};
use sp_std::convert::From;

use crate::outbound as incentivized_outbound_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		IncentivizedOutboundChannel: incentivized_outbound_channel::{Module, Call, Storage, Event},
	}
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl incentivized_outbound_channel::Config for Test {
	//type MessageCommitment = MockMessageCommitment;
	type Event = Event;
	const INDEXING_PREFIX: &'static [u8] = b"commitment";
	type Hashing = Keccak256;
}

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn test_submit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		assert_ok!(IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]));
		assert_eq!(Nonce::get(), 1);

		assert_ok!(IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]));
		assert_eq!(Nonce::get(), 2);
	});
}

// Test commitments

use frame_support::traits::OnInitialize;

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		IncentivizedOutboundChannel::on_initialize(System::block_number());
	}
}


const CONTRACT_A: H160 =  H160::repeat_byte(1);
const CONTRACT_B: H160 =  H160::repeat_byte(2);


#[test]
fn test_add_message() {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let config: incentivized_outbound_channel::GenesisConfig<Test> = incentivized_outbound_channel::GenesisConfig {
		interval: 1u64
	};
	config.assimilate_storage(&mut storage).unwrap();
	let mut t = TestExternalities::from(storage);

	t.execute_with(|| {
		System::set_block_number(1);

		IncentivizedOutboundChannel::push_message(CONTRACT_A, 0, &vec![0, 1, 2]).unwrap();
		IncentivizedOutboundChannel::push_message(CONTRACT_B, 1, &vec![3, 4, 5]).unwrap();

		let messages = vec![
			Message {
				target: CONTRACT_A,
				nonce: 0,
				payload: vec![0, 1, 2],
			},
			Message {
				target: CONTRACT_B,
				nonce: 1,
				payload: vec![3, 4, 5],
			},
		];

		assert_eq!(MessageQueue::get(), messages);

		// Run to block 5 where a commitment will be generated
		run_to_block(5);

		assert_eq!(
			System::digest().logs(),
			vec![
				DigestItem::Other(vec![0, 1, 75, 224, 75, 115, 209, 7, 157, 71, 172, 222, 139, 122, 150, 76, 83, 255, 213, 213, 15, 233, 253, 193, 12, 4, 71, 27, 94, 86, 44, 150, 225, 60])
			]
		);
	});
}
