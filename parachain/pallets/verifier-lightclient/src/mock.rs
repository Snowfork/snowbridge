// Mock runtime
use artemis_core::{Message, VerificationInput};
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

use hex_literal::hex;

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

#[derive(Clone, Eq, PartialEq)]
pub struct MockRuntimeWithPoW;

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

impl system::Trait for MockRuntimeWithPoW {
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

parameter_types! {
	pub const DescendantsUntilFinalized: u8 = 2;
	pub const PowDisabled: bool = false;
	pub const PowEnabled: bool = true;
}

impl Trait for MockRuntime {
	type Event = MockEvent;
	type DescendantsUntilFinalized = DescendantsUntilFinalized;
	type VerifyPoW = PowDisabled;
}

impl Trait for MockRuntimeWithPoW {
	type Event = MockEvent;
	type DescendantsUntilFinalized = DescendantsUntilFinalized;
	type VerifyPoW = PowEnabled;
}

pub type Verifier = Module<MockRuntime>;

pub type VerifierWithPoW = Module<MockRuntimeWithPoW>;

pub fn genesis_ethereum_header() -> EthereumHeader {
	Default::default()
}

pub fn genesis_ethereum_block_hash() -> H256 {
	genesis_ethereum_header().compute_hash()
}

pub fn child_of_genesis_ethereum_header() -> EthereumHeader {
	child_of_header(&genesis_ethereum_header())
}

pub fn child_of_header(header: &EthereumHeader) -> EthereumHeader {
	let mut child: EthereumHeader = Default::default();
	child.difficulty = 1.into();
	child.parent_hash = header.compute_hash();
	child.number = header.number + 1;
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

pub fn message_with_receipt_proof(block_hash: H256, proof: (Vec<Vec<u8>>, Vec<Vec<u8>>)) -> Message {
	Message {
		payload: Vec::new(),
		verification: VerificationInput::ReceiptProof {
			block_hash: block_hash,
			tx_index: 0,
			proof: proof,
		},
	}
}

pub fn receipt_root_and_proof() -> (H256, (Vec<Vec<u8>>, Vec<Vec<u8>>)) {
	(
		hex!("fd5e397a84884641f53c496804f24b5276cbb8c5c9cfc2342246be8e3ce5ad02").into(),
		(
			Vec::new(),
			vec!(
				hex!("f90131a0b5ba404eb5a6a88e56579f4d37ef9813b5ad7f86f0823ff3b407ac5a6bb465eca0398ead2655e78e03c127ce22c5830e90f18b1601ec055f938336c084feb915a9a026d322c26e46c50942c1aabde50e36df5cde572aed650ce73ea3182c6e90a02ca00600a356135f4db1db0d9842264cdff2652676f881669e91e316c0b6dd783011a0837f1deb4075336da320388c1edfffc56c448a43f4a5ba031300d32a7b509fc5a01c3ac82fd65b4aba7f9afaf604d9c82ec7e2deb573a091ae235751bc5c0c288da05d454159d9071b0f68b6e0503d290f23ac7602c1db0c569dee4605d8f5298f09a00bbed10350ec954448df795f6fd46e3faefc800ede061b3840eedc6e2b07a74da0acb02d26a3650f2064c14a435fdf1f668d8655daf455ebdf671713a7c089b3898080808080808080").to_vec(),
				hex!("f901f180a00046a08d4f0bdbdc6b31903086ce323182bce6725e7d9415f7ff91ee8f4820bda0e7cd26ad5f3d2771e4b5ab788e268a14a10209f94ee918eb6c829d21d3d11c1da00d4a56d9e9a6751874fd86c7e3cb1c6ad5a848da62751325f478978a00ea966ea064b81920c8f04a8a1e21f53a8280e739fbb7b00b2ab92493ca3f610b70e8ac85a0b1040ed4c55a73178b76abb16f946ce5bebd6b93ab873c83327df54047d12c27a0de6485e9ac58dc6e2b04b4bb38f562684f0b1a2ee586cc11079e7d9a9dc40b32a0d394f4d3532c3124a65fa36e69147e04fd20453a72ee9c50660f17e13ce9df48a066501003fc3e3478efd2803cd0eded6bbe9243ca01ba754d6327071ddbcbc649a0b2684e518f325fee39fc8ea81b68f3f5c785be00d087f3bed8857ae2ee8da26ea071060a5c52042e8d7ce21092f8ecf06053beb9a0b773a6f91a30c4220aa276b2a0fc22436632574ccf6043d0986dede27ea94c9ca9a3bb5ec03ce776a4ddef24a9a05a8a1d6698c4e7d8cc3a2506cb9b12ea9a079c9c7099bc919dc804033cc556e4a0170c468b0716fd36d161f0bf05875f15756a2976de92f9efe7716320509d79c9a0182f909a90cab169f3efb62387f9cccdd61440acc4deec42f68a4f7ca58075c7a055cf0e9202ac75689b76318f1171f3a44465eddc06aae0713bfb6b34fdd27b7980").to_vec(),
				hex!("f904de20b904daf904d701830652f0b9010004200000000000000000000080020000000000010000000000010000000000000000000000000000000000000000000002000000080000000000000000200000000000000000000000000008000000220000000000400010000000000000000000000000000000000000000000000000000000000000040000000010000100000000000800000000004000000000000000000000000000080000004000000000020000000000020000000000000000000000000000000000000000000004000000000002000000000100000000000000000000000000001000000002000020000010200000000000010000000000000000000000000000000000000010000000f903ccf89b9421130f34829b4c343142047a28ce96ec07814b15f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa00000000000000000000000007d843005c7433c16b27ff939cb37471541561ebda0000000000000000000000000e9c1281aae66801fa35ec404d5f2aea393ff6988a000000000000000000000000000000000000000000000000000000005d09b7380f89b9421130f34829b4c343142047a28ce96ec07814b15f863a08c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925a00000000000000000000000007d843005c7433c16b27ff939cb37471541561ebda00000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488da0ffffffffffffffffffffffffffffffffffffffffffffffffffffffcc840c6920f89b94c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000e9c1281aae66801fa35ec404d5f2aea393ff6988a00000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488da000000000000000000000000000000000000000000000000003e973b5a5d1078ef87994e9c1281aae66801fa35ec404d5f2aea393ff6988e1a01c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1b840000000000000000000000000000000000000000000000000000001f1420ad1d40000000000000000000000000000000000000000000000014ad400879d159a38f8fc94e9c1281aae66801fa35ec404d5f2aea393ff6988f863a0d78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822a00000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488da00000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488db88000000000000000000000000000000000000000000000000000000005d415f3320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e973b5a5d1078ef87a94c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2f842a07fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65a00000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488da000000000000000000000000000000000000000000000000003e973b5a5d1078e").to_vec(),
			),
		),
	)
}

pub fn new_tester() -> sp_io::TestExternalities {
	new_tester_with_config::<MockRuntime>(GenesisConfig {
		initial_header: genesis_ethereum_header(),
		initial_difficulty: 0.into(),
	})
}

pub fn new_tester_with_config<T: Trait>(config: GenesisConfig) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<T>().unwrap();

	config.assimilate_storage::<T>(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| system::Module::<T>::set_block_number(1.into()));
	ext
}
