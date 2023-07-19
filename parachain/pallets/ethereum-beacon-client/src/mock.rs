// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as ethereum_beacon_client;
use frame_support::parameter_types;
use pallet_timestamp;
use primitives::{Fork, ForkVersions};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

#[cfg(not(feature = "beacon-spec-mainnet"))]
pub mod minimal {
	use super::*;

	use crate::config;
	use std::{fs::File, path::PathBuf};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
			Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
			EthereumBeaconClient: ethereum_beacon_client::{Pallet, Call, Storage, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type OnSetCode = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_timestamp::Config for Test {
		type Moment = u64;
		type OnTimestampSet = ();
		type MinimumPeriod = ();
		type WeightInfo = ();
	}

	parameter_types! {
		pub const ExecutionHeadersPruneThreshold: u32 = 10;
		pub const ChainForkVersions: ForkVersions = ForkVersions{
			genesis: Fork {
				version: [0, 0, 0, 1], // 0x00000001
				epoch: 0,
			},
			altair: Fork {
				version: [1, 0, 0, 1], // 0x01000001
				epoch: 0,
			},
			bellatrix: Fork {
				version: [2, 0, 0, 1], // 0x02000001
				epoch: 0,
			},
			capella: Fork {
				version: [3, 0, 0, 1], // 0x03000001
				epoch: 0,
			},
		};
	}

	impl ethereum_beacon_client::Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type ForkVersions = ChainForkVersions;
		type MaxExecutionHeadersToKeep = ExecutionHeadersPruneThreshold;
		type WeightInfo = ();
	}

	// Build genesis storage according to the mock runtime.
	pub fn new_tester() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| Timestamp::set_timestamp(30_000));
		ext
	}

	fn load_fixture<T>(basename: &str) -> Result<T, serde_json::Error>
	where
		T: for<'de> serde::Deserialize<'de>,
	{
		let filepath: PathBuf =
			[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", basename].iter().collect();
		serde_json::from_reader(File::open(&filepath).unwrap())
	}

	pub fn load_execution_header_update_fixture() -> primitives::ExecutionHeaderUpdate {
		load_fixture("execution-header-update.minimal.json").unwrap()
	}

	pub fn load_checkpoint_update_fixture(
	) -> primitives::CheckpointUpdate<{ config::SYNC_COMMITTEE_SIZE }> {
		load_fixture("initial-checkpoint.minimal.json").unwrap()
	}

	pub fn load_sync_committee_update_fixture(
	) -> primitives::Update<{ config::SYNC_COMMITTEE_SIZE }, { config::SYNC_COMMITTEE_BITS_SIZE }> {
		load_fixture("sync-committee-update.minimal.json").unwrap()
	}

	pub fn load_finalized_header_update_fixture(
	) -> primitives::Update<{ config::SYNC_COMMITTEE_SIZE }, { config::SYNC_COMMITTEE_BITS_SIZE }> {
		load_fixture("finalized-header-update.minimal.json").unwrap()
	}

	pub fn load_next_sync_committee_update_fixture(
	) -> primitives::Update<{ config::SYNC_COMMITTEE_SIZE }, { config::SYNC_COMMITTEE_BITS_SIZE }> {
		load_fixture("next-sync-committee-update.minimal.json").unwrap()
	}

	pub fn load_next_finalized_header_update_fixture(
	) -> primitives::Update<{ config::SYNC_COMMITTEE_SIZE }, { config::SYNC_COMMITTEE_BITS_SIZE }> {
		load_fixture("next-finalized-header-update.minimal.json").unwrap()
	}
}

#[cfg(feature = "beacon-spec-mainnet")]
pub mod mainnet {
	use super::*;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
			Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
			EthereumBeaconClient: ethereum_beacon_client::{Pallet, Call, Storage, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type OnSetCode = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_timestamp::Config for Test {
		type Moment = u64;
		type OnTimestampSet = ();
		type MinimumPeriod = ();
		type WeightInfo = ();
	}

	parameter_types! {
		pub const ChainForkVersions: ForkVersions = ForkVersions{
			genesis: Fork {
				version: [0, 0, 16, 32], // 0x00001020
				epoch: 0,
			},
			altair: Fork {
				version: [1, 0, 16, 32], // 0x01001020
				epoch: 36660,
			},
			bellatrix: Fork {
				version: [2, 0, 16, 32], // 0x02001020
				epoch: 112260,
			},
			capella: Fork {
				version: [3, 0, 16, 32], // 0x03001020
				epoch: 162304,
			},
		};
		pub const ExecutionHeadersPruneThreshold: u32 = 10;
	}

	impl ethereum_beacon_client::Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type ForkVersions = ChainForkVersions;
		type MaxExecutionHeadersToKeep = ExecutionHeadersPruneThreshold;
		type WeightInfo = ();
	}

	// Build genesis storage according to the mock runtime.
	pub fn new_tester() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| Timestamp::set_timestamp(30_000));
		ext
	}
}
