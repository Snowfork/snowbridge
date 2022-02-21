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
	// https://simpleserialize.com/


	let attested_header: ethereum2_light_client::BeaconBlockHeader = BeaconBlockHeader{
		parent_root: hex!("bb3c91ab6ac7ad8e780d4425f9482aedaf51328f2bb9d5741aa2ca44dc6ba461").into(),
		state_root: hex!("adc929aa6a86230d2608e97ce95aa6a89e56d182b15822f4f302332e7b658312").into(),
		body_root: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		proposer_index: 175002,
		slot: 2_382_682,
	};

	let finalized_header: ethereum2_light_client::BeaconBlockHeader = BeaconBlockHeader{
		parent_root: hex!("bb3c91ab6ac7ad8e780d4425f9482aedaf51328f2bb9d5741aa2ca44dc6ba461").into(),
		state_root: hex!("adc929aa6a86230d2608e97ce95aa6a89e56d182b15822f4f302332e7b658312").into(),
		body_root: hex!("70ecb5ff5cc1c12ec223b4136cba6454eca822aca187f5c18c0d86a5c30ee5dc").into(),
		proposer_index: 175002,
		slot: 2_382_672,
	};

	let update: ethereum2_light_client::LightClientUpdate = LightClientUpdate{
		attested_header: attested_header,
		finality_branch: vec![
			hex!("3670c5c45d82686c844a30e23854f2e32cdcadf654c285998d3267d99d7d165e").into(),
			hex!("83a0ee3b3352e98d0918d59a427670261af75b7903fdfefc73a85ad39abf8b32").into(),
			hex!("731eaeba1ccf1d442b915a537f89ae9f211677535e142b5d14750e692c7a42ca").into(),
			hex!("ac5925beb9ef24aa3c84522d168ac722a83970ca92908dbdfc9d770fca5cb659").into(),
			hex!("fee14011611c7e0c3e70c264522ee739a25c82af1530533f5eeefa18787e3dad").into(),
			hex!("5d0cb03baca66860a9d039c7579503c2b0e7e9d5d8d767b007e0064ce22df0c8").into(),
		],
		finalized_header: Some(finalized_header),
		next_sync_committee: SyncCommittee{
			pubkeys: vec![
				hex!("8cf4ed7e7b07b0ffde2f1e980db08a69b185672a37b24298cfcfc6b2d689da993af4b39d0f9c58d1e44924202cbc55a7").to_vec(),
				hex!("840871eda7b20bb2a252daa1adbb0a56522e3a4fb6b739024c7e941c180043cb6fb645e04bf1a4dd0fc22b959c7aed88").to_vec(),
				hex!("aebfb94a1af08b353058c5dda6667e470574c6e25aba2a7c666f4138beffd0e9021bd48c05e83c2922b0e5d10682f8fe").to_vec(),
			],
			aggregate_pubkey: hex!("aebfb94a1af08b353058c5dda6667e470574c6e25aba2a7c666f4138beffd0e9021bd48c05e83c2922b0e5d10682f8fe").to_vec(),
		},
		next_sync_committee_branch: vec![
			hex!("488b03f278906af289ad2a8d1ee4ddc6f24b2165ecaa15e7d02ede74486a1ce1").into(),
			hex!("458e223eb0c5e08154ea7958c89a07bf270f6eac1517bf7274c37f243cb78f2b").into(),
			hex!("731eaeba1ccf1d442b915a537f89ae9f211677535e142b5d14750e692c7a42ca").into(),
			hex!("e7b9a357c75093e8fb5227159163b20755014b8e9563e5be4b5a714a38f5cf37").into(),
			hex!("dda8f3041d8687637583f222357743e0906d7b020f77782eac4eeda93b6d22bc").into(),
			hex!("47bb833d2cc75e7dc85f198fcd1a93fceaff60c960b50496d9cc88d4a2fb6dbf").into(),
		],
		pubfork_version: hex!("bfef16ac").to_vec(),
		sync_aggregate: SyncAggregate{
			sync_committee_bits: vec!(0, 1, 0, 1),
			sync_committee_signature: hex!("0bf8234d9f2462bb7523beae163a392afe715b72072e2611323fe14a4
			cd7f4e80f94ded86e468e05ca3419bdfc47ffc62f94fddf6e1590adca0d886087320ae822296088deb3f78a48
			8ebbcb3dcc1d809d00a8cbc2b5d9542c206f2a61593552").to_vec(),
		}
	};
	
	update
}
