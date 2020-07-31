// Mock runtime

use crate::{Module, Trait};
use sp_core::H256;
use frame_support::traits::StorageMapShim;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;
use balances;

impl_outer_origin! {
	pub enum Origin for MockRuntime {}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for MockRuntime {
        system<T>,
        test_events<T>,
        balances<T>,
    }
}

pub type AccountId = u64;
pub type Balance = u128;

#[derive(Clone, Eq, PartialEq)]
pub struct MockRuntime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for MockRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
}

impl balances::Trait for MockRuntime {
	type Balance = Balance;
	type Event = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = StorageMapShim<
		balances::Account<MockRuntime>,
		system::CallOnCreatedAccount<MockRuntime>,
		system::CallKillAccount<MockRuntime>,
		AccountId,
		balances::AccountData<Balance>,
	>;
}

impl Trait for MockRuntime {
	type Event = ();
	type Currency = balances::Module<MockRuntime>;
}

pub type PolkaETHModule = Module<MockRuntime>;
pub type BalancesPolkaETH = balances::Module<MockRuntime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CAROL: AccountId = 3;

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

	balances::GenesisConfig::<MockRuntime> {
		balances: vec![
			(ALICE, 1000),
			(BOB, 1000),
			(CAROL, 1000),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}
