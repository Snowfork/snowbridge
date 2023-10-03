// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as snowbridge_control;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, Currency, Contains},
	PalletId,
};
use sp_core::H256;
use xcm_executor::traits::ConvertLocation;

#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::v2::whitelisted_caller;

use snowbridge_core::outbound::{Message, MessageHash, ParaId, SubmitError};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup}, AccountId32,
};
use xcm::prelude::*;
use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription, ParentIsPreset, SiblingParachainConvertsVia, AccountId32Aliases};
use polkadot_parachain::primitives::Sibling;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = AccountId32;

// A stripped-down version of pallet-xcm that only inserts an XCM origin into the runtime
#[allow(dead_code)]
#[frame_support::pallet]
mod pallet_xcm_origin {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{OriginTrait, Contains};
	use xcm::latest::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + crate::Config {
		type RuntimeOrigin: From<Origin> + From<<Self as frame_system::Config>::RuntimeOrigin>;
	}

	// Insert this custom Origin into the aggregate RuntimeOrigin
	#[pallet::origin]
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Origin(pub MultiLocation);

	impl From<MultiLocation> for Origin {
		fn from(location: MultiLocation) -> Origin {
			Origin(location)
		}
	}

	/// `EnsureOrigin` implementation succeeding with a `MultiLocation` value to recognize and filter
	/// the contained location
	pub struct EnsureXcm<F>(PhantomData<F>);
	impl<O: OriginTrait + From<Origin>, F: Contains<MultiLocation>> EnsureOrigin<O> for EnsureXcm<F>
	where
		O::PalletsOrigin: From<Origin> + TryInto<Origin, Error = O::PalletsOrigin>,
	{
		type Success = MultiLocation;

		fn try_origin(outer: O) -> Result<Self::Success, O> {
			outer.try_with_caller(|caller| {
				caller.try_into().and_then(|o| match o {
					Origin(location) if F::contains(&location) => Ok(location),
					o => Err(o.into()),
				})
			})
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<O, ()> {
			Ok(O::from(Origin(Here.into())))
		}
	}
}

pub struct AllowSiblingsChildrenOnly;
impl Contains<MultiLocation> for AllowSiblingsChildrenOnly {
	fn contains(l: &MultiLocation) -> bool {
		match l.split_first_interior() {
			(MultiLocation { parents: 1, .. }, Some(Parachain(_))) => true,
			_ => false,

		}
	}
}

pub struct AllowSiblingsOnly;
impl Contains<MultiLocation> for AllowSiblingsOnly {
	fn contains(l: &MultiLocation) -> bool {
		match l {
			MultiLocation { parents: 1, interior: X1(Parachain(_)) } => true,
			_ => false,
		}
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		XcmOrigin: pallet_xcm_origin::{Pallet, Origin},
		EthereumControl: snowbridge_control,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

impl pallet_xcm_origin::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
}

parameter_types! {
	pub const OwnParaId: ParaId = ParaId::new(1013);
	pub const SS58Prefix: u8 = 42;
	pub const AnyNetwork: Option<NetworkId> = None;
	pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::Kusama);
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(1013));
}

pub struct MockOutboundQueue;
impl snowbridge_control::OutboundQueueTrait for MockOutboundQueue {
	type Ticket = Message;
	type Balance = u128;

	fn validate(message: &Message) -> Result<(Self::Ticket, Self::Balance), SubmitError> {
		Ok((message.clone(), 0))
	}

	fn submit(_ticket: Self::Ticket) -> Result<MessageHash, SubmitError> {
		Ok(MessageHash::zero())
	}
}

parameter_types! {
	pub TreasuryAccount: AccountId = PalletId(*b"py/trsry").into_account_truncating();
	pub Fee: u64 = 1000;
	pub const RococoNetwork: NetworkId = NetworkId::Rococo;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining sovereign accounts for asset transacting.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
	// Other nested consensus systems on sibling parachains or relay chain.
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>
);

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OwnParaId = OwnParaId;
	type OutboundQueue = MockOutboundQueue;
	type MessageHasher = BlakeTwo256;
	type AgentOrigin = pallet_xcm_origin::EnsureXcm<AllowSiblingsChildrenOnly>;
	type ChannelOrigin = pallet_xcm_origin::EnsureXcm<AllowSiblingsOnly>;
	type UniversalLocation = UniversalLocation;
	type RelayLocation = RelayLocation;
	type AgentIdOf = HashedDescription<H256, DescribeFamily<DescribeAllTerminal>>;
	type TreasuryAccount = TreasuryAccount;
	type SovereignAccountOf = LocationToAccountId;
	type Token = Balances;
	type Fee = Fee;
	type WeightInfo = ();
}

fn setup() {
	System::set_block_number(1);
	Balances::make_free_balance_be(
		&<Test as super::pallet::Config>::SovereignAccountOf::convert_location(
			&MultiLocation::parent(),
		)
		.unwrap(),
		1_000_000_000_000,
	);
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| setup());
	ext
}

// Test helpers

pub fn make_xcm_origin(location: MultiLocation) -> RuntimeOrigin {
	pallet_xcm_origin::Origin(location).into()
}

pub fn agent_id_of(location: &MultiLocation) -> Option<H256> {
	let reanchored_location = EthereumControl::reanchor_origin_location(location).unwrap();
	HashedDescription::<H256, DescribeFamily<DescribeAllTerminal>>::convert_location(&reanchored_location)
}

pub fn sovereign_account_of(location: &MultiLocation) -> Option<AccountId> {
	LocationToAccountId::convert_location(location)
}
