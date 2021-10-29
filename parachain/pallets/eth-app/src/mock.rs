use sp_std::marker::PhantomData;

// Mock runtime
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	parameter_types,
	traits::GenesisBuild,
};
use frame_system as system;
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};

use snowbridge_assets::SingleAssetAdaptor;
use snowbridge_core::{AssetId, ChannelId, OutboundRouter};

use crate as eth_app;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Assets: snowbridge_assets::{Pallet, Call, Storage, Event<T>},
		Dispatch: snowbridge_dispatch::{Pallet, Call, Storage, Origin, Event<T>},
		EthApp: eth_app::{Pallet, Call, Config, Storage, Event<T>},
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
	type OnSetCode = ();
}

impl snowbridge_assets::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

impl snowbridge_dispatch::Config for Test {
	type Origin = Origin;
	type Event = Event;
	type MessageId = u64;
	type Call = Call;
	type CallFilter = ();
}

pub struct MockOutboundRouter<AccountId>(PhantomData<AccountId>);

impl<AccountId> OutboundRouter<AccountId> for MockOutboundRouter<AccountId> {
	fn submit(channel: ChannelId, _: &AccountId, _: H160, _: &[u8]) -> DispatchResult {
		if channel == ChannelId::Basic {
			return Err(DispatchError::Other("some error!"))
		}
		Ok(())
	}
}

parameter_types! {
	pub const EthAssetId: AssetId = AssetId::ETH;
}

impl eth_app::Config for Test {
	type Event = Event;
	type Asset = Asset;
	type OutboundRouter = MockOutboundRouter<Self::AccountId>;
	type CallOrigin = snowbridge_dispatch::EnsureEthereumAccount;
	type WeightInfo = ();
}

pub type Asset = SingleAssetAdaptor<Test, EthAssetId>;

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config = eth_app::GenesisConfig { address: H160::repeat_byte(1) };
	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
