// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as snowbridge_control;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use snowbridge_core::ParaId;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use sp_runtime::AccountId32;
use xcm::v3::NetworkId;
use xcm_builder::{EnsureXcmOrigin, SignedToAccountId32};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = AccountId32;

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
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, AnyNetwork>;

impl snowbridge_control::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OwnParaId = OwnParaId;
	type OutboundQueue = ();
	type MessageHasher = BlakeTwo256;
	type MaxUpgradeDataSize = MaxUpgradeDataSize;
	type EnsureCreateAgentOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
