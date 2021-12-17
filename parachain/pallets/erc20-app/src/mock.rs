// Mock runtime
use sp_std::marker::PhantomData;

use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	traits::{Everything, GenesisBuild},
};
use frame_system as system;
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify},
	MultiSignature,
};

use snowbridge_assets::SingleAssetAdaptor;
use snowbridge_core::{assets::XcmReserveTransfer, AssetId, ChannelId};

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
		BasicOutboundChannel: snowbridge_basic_channel::outbound::{Pallet, Call, Config<T>, Storage, Event<T>},
		IncentivizedOutboundChannel: snowbridge_incentivized_channel::outbound::{Pallet, Call, Config<T>, Storage, Event<T>},
		Dispatch: snowbridge_dispatch::{Pallet, Call, Storage, Origin, Event<T>},
		Erc20App: crate::{Pallet, Call, Config, Storage, Event<T>},
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
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
	pub const Ether: AssetId = AssetId::ETH;
	pub const MaxMessagePayloadSize: u64 = 256;
	pub const MaxMessagesPerCommit: u64 = 3;
}

impl snowbridge_assets::Config for Test {
	type Event = Event;
	type WeightInfo = ();
}

impl snowbridge_basic_channel::outbound::Config for Test {
	const INDEXING_PREFIX: &'static [u8] = b"commitment";
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type SetPrincipalOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

impl snowbridge_incentivized_channel::outbound::Config for Test {
	const INDEXING_PREFIX: &'static [u8] = b"commitment";
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type FeeCurrency = SingleAssetAdaptor<Test, Ether>;
	type SetFeeOrigin = frame_system::EnsureRoot<AccountId>;
	type WeightInfo = ();
}

pub struct XcmAssetTransfererMock<T>(PhantomData<T>);
impl XcmReserveTransfer<AccountId, Origin> for XcmAssetTransfererMock<Test> {
	fn reserve_transfer(
		_origin: Origin,
		_asset_id: AssetId,
		_para_id: u32,
		_dest: &AccountId,
		_amount: ethabi::U256,
	) -> DispatchResult {
		todo!()
	}
}

impl crate::Config for Test {
	type Event = Event;
	type Assets = Assets;
	type OutboundRouter = OutboundRouter<Test>;
	type CallOrigin = snowbridge_dispatch::EnsureEthereumAccount;
	type WeightInfo = ();
	type XcmReserveTransfer = XcmAssetTransfererMock<Self>;
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::Config for Test {}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config = crate::GenesisConfig { address: H160::repeat_byte(1) };
	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
