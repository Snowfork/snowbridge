// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate as ethereum_beacon_client;
use frame_support::parameter_types;
use primitives::{Fork, ForkVersions};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

#[cfg(feature = "fuzzing")]
pub mod minimal {
    use super::*;

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    use sp_runtime::BuildStorage;

    frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
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
        sp_io::TestExternalities::new(t)
    }
}

