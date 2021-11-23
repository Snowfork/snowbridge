// Mock runtime
use sp_std::marker::PhantomData;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	parameter_types,
	traits::{Everything, GenesisBuild},
	PalletId,
};
use frame_system as system;
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};

use snowbridge_core::{ChannelId, OutboundRouter};

use crate as erc20_app;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		Dispatch: snowbridge_dispatch::{Pallet, Call, Storage, Origin, Event<T>},
		Erc20App: erc20_app::{Pallet, Call, Config, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
		pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
		pub const AssetDeposit: u64 = 1;
		pub const ApprovalDeposit: u64 = 1;
		pub const StringLimit: u32 = 50;
		pub const MetadataDepositBase: u64 = 1;
		pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u128;
	type AssetId = u128;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

impl snowbridge_dispatch::Config for Test {
	type Origin = Origin;
	type Event = Event;
	type MessageId = u64;
	type Call = Call;
	type CallFilter = Everything;
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
	pub const Erc20AppPalletId: PalletId = PalletId(*b"erc20app");
}

impl erc20_app::Config for Test {
	type Event = Event;
	type PalletId = Erc20AppPalletId;
	type Hashing = BlakeTwo256;
	type Randomness = Randomness;
	type Assets = Assets;
	type OutboundRouter = MockOutboundRouter<Self::AccountId>;
	type CallOrigin = snowbridge_dispatch::EnsureEthereumAccount;
	type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config = erc20_app::GenesisConfig { address: H160::repeat_byte(1) };
	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
