// Mock runtime

use crate::{Module, Config};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, IdentifyAccount, Verify}, testing::Header, Perbill, MultiSignature
};
use frame_system as system;

use artemis_core::AssetId;
use artemis_assets::SingleAssetAdaptor;

impl_outer_origin! {
	pub enum Origin for MockRuntime {}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum MockEvent for MockRuntime {
		system<T>,
		artemis_assets<T>,
		artemis_commitments,
        test_events<T>,
    }
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Eq, PartialEq)]
pub struct MockRuntime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Config for MockRuntime {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = MockEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl artemis_assets::Config for MockRuntime {
	type Event = MockEvent;
}

parameter_types! {
	pub const CommitInterval: u64 = 20;
}

impl artemis_commitments::Config for MockRuntime {
	type Event = MockEvent;
	type CommitInterval = CommitInterval;
}

parameter_types! {
	pub const EthAssetId: AssetId = AssetId::ETH;
}

impl Config for MockRuntime {
	type Event = MockEvent;
	type Asset = Asset;
	type Commitments = Commitments;
}

pub type System = system::Module<MockRuntime>;
pub type Commitments = artemis_commitments::Module<MockRuntime>;
pub type Asset = SingleAssetAdaptor<MockRuntime, EthAssetId>;
pub type ETH = Module<MockRuntime>;

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
