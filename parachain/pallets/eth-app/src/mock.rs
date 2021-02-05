// Mock runtime

use crate::{Module, GenesisConfig, Config};
use sp_core::{H160, H256};
use frame_support::{
	impl_outer_origin, impl_outer_event, impl_outer_dispatch, parameter_types,
	weights::Weight,
	dispatch::DispatchResult,
};
use sp_runtime::{
	traits::{
		BlakeTwo256, IdentityLookup, IdentifyAccount, Verify,
	}, testing::Header, Perbill, MultiSignature,
};
use frame_system as system;

use artemis_core::{ChannelId, AssetId, SubmitOutbound};
use artemis_assets::SingleAssetAdaptor;

use crate as eth_app;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {
		artemis_dispatch
	}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
			frame_system::System,
			artemis_assets::Assets,
			artemis_dispatch::Dispatch,
			eth_app::ETH,
	}
}

impl_outer_event! {
	pub enum Event for Test {
			system<T>,
			artemis_assets<T>,
			artemis_dispatch<T>,
			eth_app<T>,
	}
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
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
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl artemis_assets::Config for Test {
	type Event = Event;
}

impl artemis_dispatch::Config for Test {
	type Origin = Origin;
	type Event = Event;
	type MessageId = u64;
	type Call = Call;
	type CallFilter = ();
}

pub struct MockSubmitOutbound;

impl SubmitOutbound for MockSubmitOutbound {
	fn submit(_: ChannelId, _: H160, _: &[u8]) -> DispatchResult {
		Ok(())
	}
}

parameter_types! {
	pub const EthAssetId: AssetId = AssetId::ETH;
}

impl Config for Test {
	type Event = Event;
	type Asset = Asset;
	type SubmitOutbound = MockSubmitOutbound;
	type CallOrigin = artemis_dispatch::EnsureEthereumAccount;
}

pub type System = system::Module<Test>;
pub type Dispatch = artemis_dispatch::Module<Test>;
pub type Assets = artemis_assets::Module<Test>;
pub type ETH = Module<Test>;

pub type Asset = SingleAssetAdaptor<Test, EthAssetId>;

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config: GenesisConfig = GenesisConfig {
		address: H160::repeat_byte(1),
	};
	config.assimilate_storage(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
