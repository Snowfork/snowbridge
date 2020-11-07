// Mock runtime
use artemis_testutils::BlockWithProofs;
use crate::{Module, EthashProofData, EthereumHeader, GenesisConfig, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, IdentifyAccount, Verify}, testing::Header, Perbill, MultiSignature
};
use frame_system as system;
use std::fs::File;
use std::path::PathBuf;

impl_outer_origin! {
	pub enum Origin for MockRuntime {}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum MockEvent for MockRuntime {
		system<T>,
        test_events,
    }
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Eq, PartialEq)]
pub struct MockRuntime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for MockRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = MockEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type ModuleToIndex = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl Trait for MockRuntime {
	type Event = MockEvent;
}

pub type System = system::Module<MockRuntime>;
pub type Verifier = Module<MockRuntime>;

pub fn genesis_ethereum_header() -> EthereumHeader {
	Default::default()
}

pub fn genesis_ethereum_block_hash() -> H256 {
	genesis_ethereum_header().compute_hash()
}

pub fn child_of_genesis_ethereum_header() -> EthereumHeader {
	let mut child: EthereumHeader = Default::default();
	child.parent_hash = genesis_ethereum_block_hash();
	child	
}

fn fixture_path(name: &str) -> PathBuf {
	[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
}

pub fn ethereum_header_from_file(block_num: u64) -> EthereumHeader {
	let filepath = fixture_path(&format!("{}.json", block_num));
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

pub fn ethereum_header_proof_from_file(block_num: u64) -> Vec<EthashProofData> {
	let filepath = fixture_path(&format!("{}_proof.json", block_num));
	BlockWithProofs::from_file(&filepath)
		.to_double_node_with_merkle_proof_vec(EthashProofData::from_values)
}

pub fn new_tester() -> sp_io::TestExternalities {
	new_tester_with_config(GenesisConfig {
		initial_header: genesis_ethereum_header(),
		initial_difficulty: 0.into(),
		verify_pow: false,
	})
}

pub fn new_tester_with_config(config: GenesisConfig) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<MockRuntime>().unwrap();

	config.assimilate_storage::<MockRuntime>(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
