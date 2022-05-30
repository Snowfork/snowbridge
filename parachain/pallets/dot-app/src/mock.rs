// Mock runtime

use super::*;

use sp_std::marker::PhantomData;

use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	traits::{tokens::fungible::ItemOf, Everything, GenesisBuild},
	PalletId,
};
use frame_system as system;
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify},
	MultiSignature,
};

use snowbridge_core::ChannelId;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Config<T>, Storage, Event<T>},
		BasicOutboundChannel: snowbridge_basic_channel::outbound::{Pallet, Call, Config<T>, Storage, Event<T>},
		IncentivizedOutboundChannel: snowbridge_incentivized_channel::outbound::{Pallet, Call, Config<T>, Storage, Event<T>},
		Dispatch: snowbridge_dispatch::{Pallet, Call, Storage, Origin, Event<T>},
		DotApp: crate::{Pallet, Call, Config, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub type Balance = u128;

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
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u64 = 1;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
	pub const AssetAccountDeposit: u64 = 1;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u128;
	type AssetId = u128;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

parameter_types! {
	pub const EtherAssetId: u128 = 0;
	pub const EtherAppPalletId: PalletId = PalletId(*b"etherapp");
	pub const MaxMessagePayloadSize: u32 = 256;
	pub const MaxMessagesPerCommit: u32 = 3;
}

pub type Ether = ItemOf<Assets, EtherAssetId, AccountId>;

impl snowbridge_basic_channel::outbound::Config for Test {
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type SetPrincipalOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

impl snowbridge_incentivized_channel::outbound::Config for Test {
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type FeeCurrency = Ether;
	type SetFeeOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

impl snowbridge_dispatch::Config for Test {
	type Origin = Origin;
	type Event = Event;
	type MessageId = u64;
	type Call = Call;
	type CallFilter = Everything;
}

pub struct OutboundRouter<T>(PhantomData<T>);

impl<T> snowbridge_core::OutboundRouter<T::AccountId> for OutboundRouter<T>
where
	T: snowbridge_basic_channel::outbound::Config
		+ snowbridge_incentivized_channel::outbound::Config,
{
	fn submit(
		channel_id: ChannelId,
		who: &T::AccountId,
		target: H160,
		payload: &[u8],
	) -> DispatchResult {
		match channel_id {
			ChannelId::Basic =>
				snowbridge_basic_channel::outbound::Pallet::<T>::submit(who, target, payload),
			ChannelId::Incentivized =>
				snowbridge_incentivized_channel::outbound::Pallet::<T>::submit(who, target, payload),
		}
	}
}

parameter_types! {
	pub const DotPalletId: PalletId = PalletId(*b"s/dotapp");
	pub const Decimals: u32 = 12;
}

impl crate::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type OutboundRouter = OutboundRouter<Test>;
	type CallOrigin = snowbridge_dispatch::EnsureEthereumAccount;
	type PalletId = DotPalletId;
	type Decimals = Decimals;
	type WeightInfo = ();
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::Config for Test {}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config = crate::GenesisConfig { address: H160::repeat_byte(1) };
	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let assets_config: pallet_assets::GenesisConfig<Test> = pallet_assets::GenesisConfig {
		assets: vec![(0, EtherAppPalletId::get().into_account(), true, 1)],
		metadata: vec![],
		accounts: vec![],
	};
	GenesisBuild::<Test>::assimilate_storage(&assets_config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
