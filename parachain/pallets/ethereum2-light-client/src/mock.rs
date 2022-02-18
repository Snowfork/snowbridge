use super::*;
use crate as ethereum2_light_client;
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
		Ethereum2LightClient: ethereum2_light_client::{Pallet, Call, Config, Storage, Event<T>},
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

parameter_types! {
	pub const FinalizedRootIndex: u16 = 1;
	pub const NextSyncCommitteeIndex: u16 = 1;
}

impl ethereum2_light_client::Config for Test {
	type Event = Event;
	type FinalizedRootIndex = FinalizedRootIndex;
	type NextSyncCommitteeIndex = NextSyncCommitteeIndex;
}

// Build genesis storage according to the mock runtime.
pub fn new_tester() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn get_update() -> ethereum2_light_client::LightClientUpdate {
	let attested_header: ethereum2_light_client::BeaconBlockHeader = BeaconBlockHeader{
		parent_root: hex!("bb3c91ab6ac7ad8e780d4425f9482aedaf51328f2bb9d5741aa2ca44dc6ba461").into(),
		state_root: hex!("adc929aa6a86230d2608e97ce95aa6a89e56d182b15822f4f302332e7b658312").into(),
		block_root: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		proposer_index: 175002,
		slot: 2_382_682,
		signature: hex!("91f74c7ac2a43ed95b3557c7ba304f0230ffd8d2c67284c9491dabbfa562a994f3ba
		b06e88f5ce761d874b66bf7c38890d6d606fdb24c46ae3c12db815cc89b5326ca5aee5c5a7c7de2f393692
		b7f3a5e0da708451f101df63c22ee52a65f64a").to_vec()
	};

	let finalized_header: ethereum2_light_client::BeaconBlockHeader = BeaconBlockHeader{
		parent_root: hex!("bb3c91ab6ac7ad8e780d4425f9482aedaf51328f2bb9d5741aa2ca44dc6ba461").into(),
		state_root: hex!("adc929aa6a86230d2608e97ce95aa6a89e56d182b15822f4f302332e7b658312").into(),
		block_root: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		proposer_index: 175002,
		slot: 2_382_672,
		signature: hex!("91f74c7ac2a43ed95b3557c7ba304f0230ffd8d2c67284c9491dabbfa562a994f3ba
		b06e88f5ce761d874b66bf7c38890d6d606fdb24c46ae3c12db815cc89b5326ca5aee5c5a7c7de2f393692
		b7f3a5e0da708451f101df63c22ee52a65f64a").to_vec()
	};

	let update: ethereum2_light_client::LightClientUpdate = LightClientUpdate{
		attested_header: attested_header,
		finality_branch: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(), // todo fix
		finalized_header: Some(finalized_header),
		next_sync_committee: SyncCommittee{
			pubkeys: vec![
				hex!("8cf4ed7e7b07b0ffde2f1e980db08a69b185672a37b24298cfcfc6b2d689da993af4b39d0f9c58d1e44924202cbc55a7").to_vec(),
				hex!("840871eda7b20bb2a252daa1adbb0a56522e3a4fb6b739024c7e941c180043cb6fb645e04bf1a4dd0fc22b959c7aed88").to_vec(),
				hex!("aebfb94a1af08b353058c5dda6667e470574c6e25aba2a7c666f4138beffd0e9021bd48c05e83c2922b0e5d10682f8fe").to_vec(),
			]
		},
		next_sync_committee_branch: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(), // todo fix
		pubfork_version: 6,
		sync_aggregate: SyncAggregate{
			sync_committee_bits: vec!(0, 1, 0, 1),
			sync_committee_signature: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		}
	};
	
	update
}