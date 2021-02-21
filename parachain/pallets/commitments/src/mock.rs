use sp_core::H256;
use frame_support::{parameter_types};
use sp_runtime::{
	traits::{BlakeTwo256, Keccak256, IdentityLookup}, testing::Header,
};

use crate as commitments;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		Commitments: commitments::{Module, Call, Storage, Event},
	}
);


impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
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

parameter_types! {
	pub const CommitInterval: u64 = 5;
}

impl commitments::Config for Test {
	const INDEXING_PREFIX: &'static [u8] = b"commitment";
	type Event = Event;
	type Hashing = Keccak256;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let config: commitments::GenesisConfig<Test> = commitments::GenesisConfig {
		interval: 1u64
	};
	config.assimilate_storage(&mut storage).unwrap();
	storage.into()
}
