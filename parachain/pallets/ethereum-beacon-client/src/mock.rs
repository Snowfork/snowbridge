//use super::*;

use crate::config;

use crate as ethereum_beacon_client;
use frame_support::parameter_types;
use pallet_timestamp;
use primitives::{BeaconHeader, Fork, ForkVersions, PublicKey, Signature};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use std::{fs::File, path::PathBuf};

pub mod mock_minimal {
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
		pub const FinalizedHeaderPruneThreshold: u32 = 10;
		pub const SyncCommitteePruneThreshold: u32 = 4;
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
		type TimeProvider = pallet_timestamp::Pallet<Test>;
		type RuntimeEvent = RuntimeEvent;
		type ForkVersions = ChainForkVersions;
		type WeakSubjectivityPeriodSeconds = WeakSubjectivityPeriodSeconds;
		type MaxSyncCommitteesToKeep = SyncCommitteePruneThreshold;
		type MaxExecutionHeadersToKeep = ExecutionHeadersPruneThreshold;
		type MaxFinalizedHeadersToKeep = FinalizedHeaderPruneThreshold;
		type WeightInfo = ();
	}
}

pub mod mock_mainnet {
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
		pub const SyncCommitteePruneThreshold: u32 = 4;
		pub const ExecutionHeadersPruneThreshold: u32 = 10;
		pub const FinalizedHeaderPruneThreshold: u32 = 10;
	}

	impl ethereum_beacon_client::Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type TimeProvider = pallet_timestamp::Pallet<Test>;
		type ForkVersions = ChainForkVersions;
		type MaxSyncCommitteesToKeep = SyncCommitteePruneThreshold;
		type MaxExecutionHeadersToKeep = ExecutionHeadersPruneThreshold;
		type WeakSubjectivityPeriodSeconds = WeakSubjectivityPeriodSeconds;
		type MaxFinalizedHeadersToKeep = FinalizedHeaderPruneThreshold;
		type WeightInfo = ();
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_tester<T: crate::Config>() -> sp_io::TestExternalities {
	#[cfg(not(feature = "minimal"))]
	use crate::mock::mock_mainnet::Timestamp;
	#[cfg(feature = "minimal")]
	use crate::mock::mock_minimal::Timestamp;
	let t = frame_system::GenesisConfig::default().build_storage::<T>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| Timestamp::set_timestamp(30_000));
	ext
}

pub struct BLSSignatureVerifyTest<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
> {
	pub sync_committee_bits: [u8; SYNC_COMMITTEE_BITS_SIZE],
	pub sync_committee_signature: Signature,
	pub pubkeys: [PublicKey; SYNC_COMMITTEE_SIZE],
	pub header: BeaconHeader,
	pub validators_root: H256,
	pub signature_slot: u64,
}

fn fixture_path(name: &str) -> PathBuf {
	[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
}

fn initial_sync_from_file<const SYNC_COMMITTEE_SIZE: usize>(
	name: &str,
) -> primitives::CheckPointUpdate<SYNC_COMMITTEE_SIZE> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn sync_committee_update_from_file<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>(
	name: &str,
) -> primitives::SyncCommitteeUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn sync_committee_from_file<const SYNC_COMMITTEE_SIZE: usize>(
	name: &str,
) -> primitives::SyncCommittee<SYNC_COMMITTEE_SIZE> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn finalized_header_update_from_file<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>(
	name: &str,
) -> primitives::FinalizedHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn header_update_from_file<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>(
	name: &str,
) -> primitives::ExecutionHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn get_config_setting() -> String {
	return match config::IS_MINIMAL {
		true => "minimal".to_owned(),
		false => "mainnet".to_owned(),
	}
}

fn add_file_prefix(name: &str) -> String {
	let prefix = get_config_setting();

	let mut result = prefix.to_owned();
	result.push_str("_");
	result.push_str(name);
	result
}

pub fn get_initial_sync<const SYNC_COMMITTEE_SIZE: usize>(
) -> primitives::CheckPointUpdate<SYNC_COMMITTEE_SIZE> {
	initial_sync_from_file::<SYNC_COMMITTEE_SIZE>(&add_file_prefix("initial_sync.json"))
}

pub fn get_committee_sync_period_update<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>() -> primitives::SyncCommitteeUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	sync_committee_update_from_file::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(
		&add_file_prefix("sync_committee_update.json"),
	)
}

pub fn get_committee_sync_ssz_test_data<const SYNC_COMMITTEE_SIZE: usize>(
) -> primitives::SyncCommittee<SYNC_COMMITTEE_SIZE> {
	let mut filename: String = "ssz_test_".to_owned();
	filename.push_str(&get_config_setting());
	filename.push_str("_sync_committee.json");
	sync_committee_from_file::<SYNC_COMMITTEE_SIZE>(filename.as_str())
}

pub fn get_header_update<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>() -> primitives::ExecutionHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	header_update_from_file::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(&add_file_prefix(
		"header_update.json",
	))
}

pub fn get_finalized_header_update<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>() -> primitives::FinalizedHeaderUpdate<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	finalized_header_update_from_file::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>(
		&add_file_prefix("finalized_header_update.json"),
	)
}

pub fn get_validators_root<const SYNC_COMMITTEE_SIZE: usize>() -> H256 {
	get_initial_sync::<SYNC_COMMITTEE_SIZE>().validators_root
}

pub fn get_bls_signature_verify_test_data<
	const SYNC_COMMITTEE_SIZE: usize,
	const SYNC_COMMITTEE_BITS_SIZE: usize,
>() -> BLSSignatureVerifyTest<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE> {
	let finalized_update =
		get_finalized_header_update::<SYNC_COMMITTEE_SIZE, SYNC_COMMITTEE_BITS_SIZE>();
	let initial_sync = get_initial_sync::<SYNC_COMMITTEE_SIZE>();

	BLSSignatureVerifyTest {
		sync_committee_bits: finalized_update.sync_aggregate.sync_committee_bits,
		sync_committee_signature: finalized_update.sync_aggregate.sync_committee_signature,
		pubkeys: initial_sync.current_sync_committee.pubkeys,
		header: finalized_update.attested_header,
		validators_root: initial_sync.validators_root,
		signature_slot: finalized_update.signature_slot,
	}
}
