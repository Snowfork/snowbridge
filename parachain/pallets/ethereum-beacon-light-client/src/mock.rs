use super::*;
use crate as ethereum_beacon_light_client;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, 
	testing::Header,
};
use frame_system as system;
use hex_literal::hex;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		EthereumBeaconLightClient: ethereum_beacon_light_client::{Pallet, Call, Config, Storage, Event<T>},
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
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

impl ethereum_beacon_light_client::Config for Test {
	type Event = Event;
}

// Build genesis storage according to the mock runtime.
pub fn new_tester() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn get_initial_sync() -> ethereum_beacon_light_client::LightClientInitialSync {
	let header: ethereum_beacon_light_client::BeaconBlockHeader = BeaconBlockHeader{
		parent_root: hex!("25a7daa40132591c54af55694ff37f0c481e04734af0f2b474108770d487d862").into(),
		state_root: hex!("8aeaf51cd840eacfe8d1c171f128dffd22a633836c30447aaf0bf291494526c3").into(),
		body_root: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		proposer_index: 72521,
		slot: 3469984,
	};

	let update: ethereum_beacon_light_client::LightClientInitialSync = LightClientInitialSync{
		header: header,
		current_sync_committee: SyncCommittee { 
			pubkeys: vec![
				hex!("adb6aa3a8cb9da8bef033b4a85a54df4f37f145545bc436a0726537204ff227a").into(),
				hex!("4db7cb4de5a225d4a6b8821aa75115fc1c9f7f175654b64ab1a06375b08c9e28").into(),
				hex!("c362f591ecc5f68c22589d5da9f3b3eb5c1d47d8cafa538db76b294114a8c26a").into(),
				hex!("c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c").into(),
				hex!("aefade2ca369bb39430468a43de0c68d34efcb24a9edcd262df314936b66ca05").into(),
			], 
			aggregate_pubkey: hex!("4db7cb4de5a225d4a6b8821aa75115fc1c9f7f175654b64ab1a06375b08c9e28").into(),
		},
		current_sync_committee_branch: SyncCommitteeBranch {branch: vec![
			hex!("adb6aa3a8cb9da8bef033b4a85a54df4f37f145545bc436a0726537204ff227a").into(),
			hex!("4db7cb4de5a225d4a6b8821aa75115fc1c9f7f175654b64ab1a06375b08c9e28").into(),
			hex!("c362f591ecc5f68c22589d5da9f3b3eb5c1d47d8cafa538db76b294114a8c26a").into(),
			hex!("c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c").into(),
			hex!("aefade2ca369bb39430468a43de0c68d34efcb24a9edcd262df314936b66ca05").into(),
		],
		}
	};
	
	update
}
