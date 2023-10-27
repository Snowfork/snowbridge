// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as snowbridge_control;
use frame_support::{
	parameter_types,
	traits::{tokens::fungible::Mutate, ConstU128, ConstU16, ConstU64, Contains},
	PalletId,
};
use sp_core::H256;
use xcm_executor::traits::ConvertLocation;

use snowbridge_core::{
	outbound::{Message, MessageHash, ParaId, SendError},
	AgentId,
};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	AccountId32,
};
use xcm::prelude::*;
use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription};

#[cfg(feature = "runtime-benchmarks")]
use crate::BenchmarkHelper;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;

pub type AccountId = AccountId32;

// A stripped-down version of pallet-xcm that only inserts an XCM origin into the runtime
#[allow(dead_code)]
#[frame_support::pallet]
mod pallet_xcm_origin {
	use frame_support::{
		pallet_prelude::*,
		traits::{Contains, OriginTrait},
	};
	use xcm::latest::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
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

	/// `EnsureOrigin` implementation succeeding with a `MultiLocation` value to recognize and
	/// filter the contained location
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
			Ok(O::from(Origin(MultiLocation { parents: 1, interior: X1(Parachain(2000)) })))
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
	type AccountData = pallet_balances::AccountData<Balance>;
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
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
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
	type Balance = Balance;

	fn validate(message: &Message) -> Result<(Self::Ticket, Self::Balance), SendError> {
		Ok((message.clone(), 10))
	}

	fn submit(_ticket: Self::Ticket) -> Result<MessageHash, SendError> {
		Ok(MessageHash::zero())
	}
}

parameter_types! {
	pub TreasuryAccount: AccountId = PalletId(*b"py/trsry").into_account_truncating();
	pub Fee: u64 = 1000;
	pub const RococoNetwork: NetworkId = NetworkId::Rococo;
}

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<RuntimeOrigin> for () {
	fn make_xcm_origin(location: MultiLocation) -> RuntimeOrigin {
		RuntimeOrigin::from(pallet_xcm_origin::Origin(location))
	}
}

pub struct AllowSiblingsOnly;
impl Contains<MultiLocation> for AllowSiblingsOnly {
	fn contains(location: &MultiLocation) -> bool {
		if let MultiLocation { parents: 1, interior: X1(Parachain(_)) } = location {
			true
		} else {
			false
		}
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OwnParaId = OwnParaId;
	type OutboundQueue = MockOutboundQueue;
	type MessageHasher = BlakeTwo256;
	type SiblingOrigin = pallet_xcm_origin::EnsureXcm<AllowSiblingsOnly>;
	type AgentIdOf = HashedDescription<AgentId, DescribeFamily<DescribeAllTerminal>>;
	type TreasuryAccount = TreasuryAccount;
	type Token = Balances;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| {
		System::set_block_number(1);
		let _ = Balances::mint_into(&AccountId32::from([0; 32]), 1_000_000_000_000);
	});
	ext
}

// Test helpers

pub fn make_xcm_origin(location: MultiLocation) -> RuntimeOrigin {
	pallet_xcm_origin::Origin(location).into()
}

pub fn make_agent_id(location: MultiLocation) -> AgentId {
	HashedDescription::<AgentId, DescribeFamily<DescribeAllTerminal>>::convert_location(&location)
		.expect("convert location")
}
