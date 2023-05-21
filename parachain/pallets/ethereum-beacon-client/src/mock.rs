use crate as ethereum_beacon_client;
use frame_support::parameter_types;
use pallet_timestamp;
use primitives::{Fork, ForkVersions};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

#[cfg(not(feature = "beacon-spec-mainnet"))]
pub mod minimal {
	use super::*;

	use crate::config;
	use std::{format, fs::File, path::PathBuf};

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

	fn fixture_path(name: &str) -> PathBuf {
		[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
	}

	fn initial_sync_from_file<const SYNC_COMMITTEE_SIZE: usize>(
		name: &str,
	) -> primitives::CheckpointUpdate<SYNC_COMMITTEE_SIZE> {
		let filepath = fixture_path(name);
		serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
	}

	fn update_from_file<const SYNC_COMMITTEE_SIZE: usize, const SYNC_COMMITTEE_BITS_SIZE: usize>(
		name: &str,
	) -> primitives::Update<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
		let filepath = fixture_path(name);
		serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
	}

	fn header_update_from_file(name: &str) -> primitives::ExecutionHeaderUpdate {
		let filepath = fixture_path(name);
		serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
	}

	fn beacon_spec() -> String {
		match config::IS_MINIMAL {
			true => "minimal".to_owned(),
			false => "mainnet".to_owned(),
		}
	}

	pub fn get_initial_sync<const SYNC_COMMITTEE_SIZE: usize>(
	) -> primitives::CheckpointUpdate<SYNC_COMMITTEE_SIZE> {
		initial_sync_from_file::<SYNC_COMMITTEE_SIZE>(&format!(
			"initial-checkpoint.{}.json",
			beacon_spec()
		))
	}

	pub fn get_committee_sync_period_update<
		const SYNC_COMMITTEE_SIZE: usize,
		const SYNC_COMMITTEE_BITS_SIZE: usize,
	>() -> primitives::Update<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
		update_from_file::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(&format!(
			"sync-committee-update.{}.json",
			beacon_spec()
		))
	}

	pub fn get_header_update() -> primitives::ExecutionHeaderUpdate {
		header_update_from_file(&format!("execution-header-update.{}.json", beacon_spec()))
	}

	pub fn get_finalized_header_update<
		const SYNC_COMMITTEE_SIZE: usize,
		const SYNC_COMMITTEE_BITS_SIZE: usize,
	>() -> primitives::Update<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
		update_from_file::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(&format!(
			"finalized-header-update.{}.json",
			beacon_spec()
		))
	}

	pub fn get_validators_root<const SYNC_COMMITTEE_SIZE: usize>() -> H256 {
		get_initial_sync::<SYNC_COMMITTEE_SIZE>().validators_root
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
		pub const WeakSubjectivityPeriodSeconds: u32 = 97200;
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
