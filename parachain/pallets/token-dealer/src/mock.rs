// Mock runtime

use crate::{Module, Config};
use sp_core::{H160, H256};
use frame_support::{
	impl_outer_origin, impl_outer_event, impl_outer_dispatch, parameter_types,
	traits::Get,
	weights::Weight,
};
use sp_runtime::{
	traits::{Convert, BlakeTwo256, IdentityLookup, IdentifyAccount, Verify}, testing::Header, Perbill, MultiSignature
};
use frame_system as system;

use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId};
use xcm_builder::{
	AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::{traits::NativeAsset, Config, XcmExecutor};
use cumulus_primitives::relay_chain::Balance as RelayChainBalance;

use artemis_xcm_support::Transactor;

impl_outer_origin! {
	pub enum Origin for MockRuntime {
		cumulus_message_broker,
	}
}

impl_outer_dispatch! {
	pub enum Call for MockRuntime where origin: Origin {
		pallet_balances::DOT,
		cumulus_message_broker::MessageBroker,
	}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum MockEvent for MockRuntime {
		system<T>,
		artemis_assets<T>,
		pallet_balances<T>,
		cumulus_message_broker<T>,
        test_events<T>,
    }
}

pub type Balance = u128;

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

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const TransferFee: u128 = 0;
	pub const CreationFee: u128 = 0;
	pub const TransactionByteFee: u128 = 1;
}

impl pallet_balances::Config for MockRuntime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = MockEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl artemis_assets::Config for MockRuntime {
	type Event = MockEvent;
	type AssetId = H160;
}


// Cumulus and XCMP

impl cumulus_message_broker::Config for MockRuntime {
	type DownwardMessageHandlers = ();
	type HrmpMessageHandlers = ();
}

impl parachain_info::Config for MockRuntime {}

pub struct NativeToRelay;
impl Convert<Balance, RelayChainBalance> for NativeToRelay {
	fn convert(val: u128) -> Balance {
		val
	}
}

parameter_types! {
	pub const PolkadotNetworkId: NetworkId = NetworkId::Polkadot;
}

pub struct AccountId32Converter;
impl Convert<AccountId, [u8; 32]> for AccountId32Converter {
	fn convert(account_id: AccountId) -> [u8; 32] {
		account_id.into()
	}
}

parameter_types! {
	pub ArtemisNetwork: NetworkId = NetworkId::Named("artemis".into());
	pub RelayChainOrigin: Origin = cumulus_message_broker::Origin::Relay.into();
	pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
		id: parachain_info::Module::<MockRuntime>::get().into(),
	});
}

pub type LocationConverter = (
	ParentIsDefault<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<ArtemisNetwork, AccountId>,
);

pub type LocalOriginConverter = (
	SovereignSignedViaLocation<LocationConverter, Origin>,
	RelayChainAsNative<RelayChainOrigin, Origin>,
	SiblingParachainAsNative<cumulus_message_broker::Origin, Origin>,
	SignedAccountId32AsNative<ArtemisNetwork, Origin>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = MessageBroker;
	type AssetTransactor = Transactor<DOT, Assets, LocationConverter, AccountId>;
	type OriginConverter = LocalOriginConverter;
	type IsReserve = NativeAsset;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
}


impl Config for MockRuntime {
	type Event = MockEvent;
	type Balance = Balance;
	type ToRelayChainBalance = NativeToRelay;
	type AccountIdConverter = LocationConverter;
	type AccountId32Converter = AccountId32Converter;
	type RelayChainNetworkId = PolkadotNetworkId;
	type ParaId = parachain_info::Module<MockRuntime>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type System = system::Module<MockRuntime>;
pub type DOT = pallet_balances::Module<MockRuntime>;
pub type Assets = artemis_assets::Module<MockRuntime>;
pub type MessageBroker = cumulus_message_broker::Module<MockRuntime>;

pub type TokenDealer = Module<MockRuntime>;


pub fn new_tester() -> sp_io::TestExternalities {
	let storage = system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
