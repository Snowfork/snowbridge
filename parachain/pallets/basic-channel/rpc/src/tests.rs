use assert_matches::assert_matches;
use codec::Encode;
use hex_literal::hex;
use sp_core::{H160, H256};
use sp_core::offchain::testing::{self, TestPersistentOffchainDB};
use frame_support::{assert_ok, parameter_types};
use sp_runtime::{
	traits::{BlakeTwo256, Keccak256, IdentityLookup},
	testing::Header,
};
use artemis_basic_channel::outbound as outbound_channel;

type AccountId = u64;



#[test]
fn local_storage_should_work() {
	use super::*;

	let storage = TestPersistentOffchainDB::new();
	let channel = BasicChannel::<_, AccountId>::new(storage, DenyUnsafe::No, b"testing");
	let root = H256::repeat_byte(1);
	let key = offchain_key(channel.indexing_prefix, root);
	let account_id = 1234u64;

	let value = CommitmentData{
		messages: Vec::new(),
		subcommitments: vec![(account_id, b"some_data".to_vec())],
	};
	let value = value.encode();

	channel.storage.write().set(sp_offchain::STORAGE_PREFIX, &*key, &*value.clone());

	assert!(channel.get_merkle_proofs(root).is_ok());

	assert_matches!(
		channel.get_merkle_proofs(root),
		Ok(ref proofs) if proofs[0].0 == account_id
	);

	let root2 = H256::from_slice(&hex!["0aaaaaaaaaaaaaaabbbbbbbbbbbbbbbbbcccccccccccccccdddddddddddddddd"]); // = Bytes(b"offchain_storage".to_vec());
	assert!(!channel.get_merkle_proofs(root2).is_ok());
}

#[test]
fn offchain_calls_considered_unsafe() {
	use super::*;

	let storage = TestPersistentOffchainDB::new();
	let channel = BasicChannel::<_, AccountId>::new(storage, DenyUnsafe::Yes, b"testing");
	let root = H256::repeat_byte(2);
	let key = offchain_key(channel.indexing_prefix, root);
	let account_id = 1234u64;

	let value = CommitmentData{
		messages: Vec::new(),
		subcommitments: vec![(account_id, b"some_data".to_vec())],
	};
	let value = value.encode();

	channel.storage.write().set(sp_offchain::STORAGE_PREFIX, &*key, &*value.clone());

	assert_matches!(
		channel.get_merkle_proofs(root),
	    	Err(jsonrpc_core::Error{..})
	);
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		BasicOutboundChannel: outbound_channel::{Module, Call, Storage, Event},
	}
);


impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

const INDEXING_PREFIX: &'static [u8] = b"commitment";

impl outbound_channel::Config for Test {
	type Event = Event;
	const INDEXING_PREFIX: &'static [u8] = INDEXING_PREFIX;
	type Hashing = Keccak256;
}

fn run_to_block(n: u64) {
	use crate::tests::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OnInitialize;
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		BasicOutboundChannel::on_initialize(System::block_number());
	}
}

use sp_core::offchain::OffchainExt;
use sp_io::TestExternalities;

#[test]
fn test_commit_and_read() {
	use super::*;

	let (offchain, _offchain_state) = testing::TestOffchainExt::new();

	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let config: outbound_channel::GenesisConfig<Test> = outbound_channel::GenesisConfig {
		interval: 1u64
	};
	config.assimilate_storage(&mut storage).unwrap();

	let mut t = TestExternalities::from(storage);
	t.register_extension(OffchainExt::new(offchain));

	t.execute_with(|| {
		const CONTRACT_A: H160 =  H160::repeat_byte(1);
		const CONTRACT_B: H160 =  H160::repeat_byte(2);
		const WHO: u64 = 5555;

		let messages = vec![
			outbound_channel::Message {
				target: CONTRACT_A,
				nonce: 0,
				payload: vec![0, 1, 2],
			},
			outbound_channel::Message {
				target: CONTRACT_B,
				nonce: 1,
				payload: vec![3, 3, 3],
			},
		];

		assert_ok!(BasicOutboundChannel::submit(&WHO, CONTRACT_A, &messages[0].payload));
		assert_ok!(BasicOutboundChannel::submit(&WHO, CONTRACT_B, &messages[1].payload));

		run_to_block(2);
	});

	let channel = BasicChannel::<_, AccountId>::new(t.offchain_db(), DenyUnsafe::No, INDEXING_PREFIX);
	let mroot = H256::from_slice(&hex!["b844173465763db27a0006c077aa14d8d089cb4d9a6f8a903754664f0bbe6bdd"][..]);
	let key = offchain_key(channel.indexing_prefix, mroot);
	let data = t.overlayed_changes().clone().offchain_drain_committed()
 		.find(|(k, _v)| {
 			k == &(sp_core::offchain::STORAGE_PREFIX.to_vec(), key.clone())
 		});

	use sp_core::offchain::OffchainOverlayedChange;
	assert_matches!(data.map(|data| data.1), Some(OffchainOverlayedChange::SetValue(_)));
}
