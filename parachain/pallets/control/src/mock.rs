// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as snowbridge_control;
use frame_support::{
	pallet_prelude::EnsureOrigin,
	parameter_types,
	traits::{ConstU16, ConstU64, OriginTrait},
};

#[cfg(feature = "runtime-benchmarks")]
use frame_benchmarking::v2::whitelisted_caller;

use snowbridge_core::outbound::{Message, MessageHash, ParaId, SubmitError};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use xcm::prelude::*;
use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = AccountId32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const OwnParaId: ParaId = ParaId::new(1013);
	pub const MaxUpgradeDataSize: u32 = 1024;
	pub const SS58Prefix: u8 = 42;
	pub const AnyNetwork: Option<NetworkId> = None;
	pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::Kusama);
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(1013));
}

static ORIGIN_TABLE: &[([u8; 32], MultiLocation)] = &[
	// Case 1: Bridge hub
	([1; 32], MultiLocation { parents: 0, interior: Here }),
	// Case 2: Local AccountId32
	(
		[2; 32],
		MultiLocation {
			parents: 0,
			interior: X1(Junction::AccountId32 { network: None, id: [0; 32] }),
		},
	),
	// Case 3: Local AccountKey20
	(
		[3; 32],
		MultiLocation {
			parents: 0,
			interior: X1(Junction::AccountKey20 { network: None, key: [0; 20] }),
		},
	),
	// Case 4: Local Pallet
	([4; 32], MultiLocation { parents: 0, interior: X1(Junction::PalletInstance(1)) }),
	// Case 5: Sibling Chain
	([5; 32], MultiLocation { parents: 1, interior: X1(Junction::Parachain(1000)) }),
	// Case 6: Sibling Chain Pallet
	(
		[6; 32],
		MultiLocation {
			parents: 1,
			interior: X2(Junction::Parachain(1000), Junction::PalletInstance(1)),
		},
	),
	// Case 7: Sibling Chain AccountId32
	(
		[7; 32],
		MultiLocation {
			parents: 1,
			interior: X2(
				Junction::Parachain(1000),
				Junction::AccountId32 { network: None, id: [0; 32] },
			),
		},
	),
	// Case 8: Sibling Chain AccountKey20
	(
		[8; 32],
		MultiLocation {
			parents: 1,
			interior: X2(
				Junction::Parachain(1000),
				Junction::AccountKey20 { network: None, key: [0; 20] },
			),
		},
	),
	// Case 9: Bad Multi Locations
	(
		[9; 32],
		MultiLocation {
			parents: 1,
			interior: X2(Junction::Parachain(1000), Junction::Parachain(1000)),
		},
	),
	// Case 10: Bad Validate Message
	([10; 32], MultiLocation { parents: 1, interior: X1(Junction::Parachain(1001)) }),
	// Case 11: Bad Submit Message
	([11; 32], MultiLocation { parents: 1, interior: X1(Junction::Parachain(1002)) }),
];

pub struct EnsureOriginFromTable;
impl EnsureOrigin<RuntimeOrigin> for EnsureOriginFromTable {
	type Success = MultiLocation;

	fn try_origin(outer: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		let account = outer.clone().into_signer().ok_or(outer.clone())?;

		// Benchmarking
		#[cfg(feature = "runtime-benchmarks")]
		{
			if account == whitelisted_caller() {
				return Ok(MultiLocation::new(0, Here))
			}
		}

		// test cases
		let key: [u8; 32] = account.into();
		for entry in ORIGIN_TABLE {
			if entry.0 == key {
				return Ok(entry.1)
			}
		}
		Err(outer)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::signed([0u8; 32].into()))
	}
}

pub struct MockOutboundQueue;
impl snowbridge_control::OutboundQueueTrait for MockOutboundQueue {
	type Ticket = Message;

	fn validate(message: &Message) -> Result<Self::Ticket, SubmitError> {
		Ok(message.clone())
	}

	fn submit(_ticket: Self::Ticket) -> Result<MessageHash, SubmitError> {
		Ok(MessageHash::zero())
	}
}

impl snowbridge_control::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OwnParaId = OwnParaId;
	type OutboundQueue = MockOutboundQueue;
	type MessageHasher = BlakeTwo256;
	type MaxUpgradeDataSize = MaxUpgradeDataSize;
	type ControlOrigin = EnsureOriginFromTable;
	type UniversalLocation = UniversalLocation;
	type RelayLocation = RelayLocation;
	type AgentHashedDescription = HashedDescription<H256, DescribeFamily<DescribeAllTerminal>>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
