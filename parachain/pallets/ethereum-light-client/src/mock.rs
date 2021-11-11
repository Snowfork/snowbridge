// Mock runtime
use crate::{EthashProofData, EthereumDifficultyConfig, EthereumHeader};
use frame_support::{
	parameter_types,
	traits::{Everything, GenesisBuild},
};
use frame_system as system;
use snowbridge_core::{Message, Proof};
use snowbridge_testutils::BlockWithProofs;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use std::{fs::File, path::PathBuf};

use hex_literal::hex;

use crate as verifier;

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub mod mock_verifier {

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
			Verifier: verifier::{Pallet, Call, Config, Storage, Event<T>},
		}
	);

	impl frame_system::Config for Test {
		type BaseCallFilter = Everything;
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
		type OnSetCode = ();
	}

	parameter_types! {
		pub const DescendantsUntilFinalized: u8 = 2;
		pub const DifficultyConfig: EthereumDifficultyConfig = EthereumDifficultyConfig::mainnet();
		pub const VerifyPoW: bool = false;
		pub const MaxHeadersForNumber: u32 = 10;
	}

	impl verifier::Config for Test {
		type Event = Event;
		type DescendantsUntilFinalized = DescendantsUntilFinalized;
		type DifficultyConfig = DifficultyConfig;
		type VerifyPoW = VerifyPoW;
		type WeightInfo = ();
		type MaxHeadersForNumber = MaxHeadersForNumber;
	}
}

pub mod mock_verifier_with_pow {

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
			Verifier: verifier::{Pallet, Call, Config, Storage, Event<T>},
		}
	);

	impl system::Config for Test {
		type BaseCallFilter = Everything;
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
		type OnSetCode = ();
	}

	parameter_types! {
		pub const DescendantsUntilFinalized: u8 = 2;
		pub const DifficultyConfig: EthereumDifficultyConfig = EthereumDifficultyConfig::mainnet();
		pub const VerifyPoW: bool = true;
		pub const MaxHeadersForNumber: u32 = 10;
	}

	impl verifier::Config for Test {
		type Event = Event;
		type DescendantsUntilFinalized = DescendantsUntilFinalized;
		type DifficultyConfig = DifficultyConfig;
		type VerifyPoW = VerifyPoW;
		type WeightInfo = ();
		type MaxHeadersForNumber = MaxHeadersForNumber;
	}
}

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

pub fn ropsten_london_header() -> EthereumHeader {
	EthereumHeader {
		parent_hash: hex!("1026708dc4e90f80898044cd5dcab4f225cc59edd4575c7222a792828d15789c").into(),
		timestamp: 1629371367u64.into(),
		number: 10867486u64.into(),
		author: hex!("9ffed2297c7b81293413550db675073ab46980b2").into(),
		transactions_root: hex!("d07d104a6f4f0093230be0bfbb69aaa34f7fcf8e84e804b5dc0d12229db2b1f2").into(),
		ommers_hash: hex!("7f93fe6355b0eeb0c419668cabbbc5ecb42bb9687860293d21d0c1e13f3189be").into(),
		extra_data: hex!("d883010a07846765746888676f312e31362e35856c696e7578").into(),
		state_root: hex!("f09fdc13472edc2567917840393e16ca6b215074bba792b26d23217b8ccb726b").into(),
		receipts_root: hex!("587bceddb4e618b754faf26ab09b1b10fbf957dfc6f0f79207d73e23c4324af9").into(),
		logs_bloom: (&hex!("102008480008800200004000800000000420001040004002000100110000054000008002000000000000002032000000000000002000000000200102086400800c2900000010420000000008400142240008070090048000006400008020000c0800000002000080010020000000080810800000002c001030442010280000000001810808040001f040000000480400000400010b020008400000411004000002002100002090004801080001000008002400000008000000240000000402020090005200040401400000000002000000100020d002801001002020020860820110002000240000400208020102002102022010002000c80800000000000000")).into(),
		gas_used: 6102147u64.into(),
		gas_limit: 8000000u64.into(),
		difficulty: 1578581203u64.into(),
		seal: vec![
				hex!("a050a8258722673f95e2bc00fd71643b77d9821858da66fa5075012f48d1fa0cf2").to_vec(),
				hex!("88dba6cb08cbe4337b").to_vec(),
		],
		base_fee: Some(7u64.into())
	}
}

fn fixture_path(name: &str) -> PathBuf {
	[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
}

pub fn ethereum_header_from_file(block_num: u64, suffix: &str) -> EthereumHeader {
	let filepath = fixture_path(&format!("{}{}.json", block_num, suffix));
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

pub fn ethereum_header_proof_from_file(block_num: u64, suffix: &str) -> Vec<EthashProofData> {
	let filepath = fixture_path(&format!("{}{}_proof.json", block_num, suffix));
	BlockWithProofs::from_file(&filepath)
		.to_double_node_with_merkle_proof_vec(EthashProofData::from_values)
}

pub fn message_with_receipt_proof(
	payload: Vec<u8>,
	block_hash: H256,
	proof_data: (Vec<Vec<u8>>, Vec<Vec<u8>>),
) -> Message {
	Message { data: payload, proof: Proof { block_hash, tx_index: 0, data: proof_data } }
}

// from https://ropsten.etherscan.io/tx/0x3541903322b74942aa9dd436ac6277d36d874865c35032fe915518d2659fc64c
pub fn ropsten_london_message() -> Message {
	Message {
		data: hex!("f90119945dd2b8d6f10623426b74d7a92d322f75b74571a3e1a0779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15bb8e000000000000000000000000000273e201ffb0bccce44560454fb6841429d50710000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000574101ef42cf85be6adf3081ada73af87e27996046fe6300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d000014bbf08ac602000000000000000000000000000000000000000000000000000000000000000000").to_vec(),
		proof: Proof {
			block_hash: hex!("a5b871f284c883a67a525e8001a106463234dd968c49eeb300d9382d64f25619").into(),
			tx_index: 25,
			data: (
				vec![
					hex!("587bceddb4e618b754faf26ab09b1b10fbf957dfc6f0f79207d73e23c4324af9").to_vec(),
					hex!("e94a68e76d4bb10270ee9f1f50a4155a96ca51bc1f35328113d4c201a25dd8dd").to_vec(),
					hex!("a90abaedf9feb13afac23e55b89961fc795048f32ceb6502bb94eead9f361a08").to_vec(),
				],
				vec![
					hex!("f871a01392e60e279b56496b25be598f4c7206038bf800589a7c30e86d71554fa41ee9a0e94a68e76d4bb10270ee9f1f50a4155a96ca51bc1f35328113d4c201a25dd8dd808080808080a0e58215be848c1293dd381210359d84485553000a82b67410406d183b42adbbdd8080808080808080").to_vec(),
					hex!("f90151a03313afdd3bcd74ac9ff430a1739bf4a5a32c4140ee06855ba3e98afb65290bdca06627246dc7d76a237549835b0f6adc480fc1069e9f2bde21d3fc44ab80798695a00ba55e9264f81f6a217f1af33729948632f5cdf04ff2143ef2e952a9462e7bf5a055b2d9e14ac5cbc6871328e071b405ac9a6cf8dad2fa2455ab4a8b960085e441a0d6634b5e368571ef8fc977db39865006084d57a8c27e2ec5ce43851db514ba21a07ef093602d5faf3881949f823cd8f9b65579c73f4578a8b8849316b21ad41308a0edef934e89bb383ea29b324a5744160a706cfd6de092bdb3b7c7ed376016fa89a0fc2e5f1471d608133309328cc7e3dfa6a570c7f1d734f7c7e06dee2bf53f9d47a0531dc644c4c40b4f605b4fdde692b8c180fd6c060f17499fa34a6ffc9a5de8c5a0a90abaedf9feb13afac23e55b89961fc795048f32ceb6502bb94eead9f361a0880808080808080").to_vec(),
					hex!("f902ca20b902c602f902c201835d1c83b9010000000008000080000000000000000000000000000000400000000000000000000000000000000000000000000200000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000020000000000000000000000000000000000000000000000000000000020000000000000000000000000000000100000000004000000000000000000000000000000000000000000000800000000000000000000400000000000000000000000000000000000000000000000f901b7f8999400273e201ffb0bccce44560454fb6841429d5071e1a0caae0f5e72020d428da73a237d1f9bf162e158dda6d4908769b8b60c095b01f4b860000000000000000000000000ef42cf85be6adf3081ada73af87e27996046fe63d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000000000000000000000000000002c68af0bb140000f90119945dd2b8d6f10623426b74d7a92d322f75b74571a3e1a0779b38144a38cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15bb8e000000000000000000000000000273e201ffb0bccce44560454fb6841429d50710000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000574101ef42cf85be6adf3081ada73af87e27996046fe6300d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d000014bbf08ac602000000000000000000000000000000000000000000000000000000000000000000").to_vec(),
				],
			),
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

pub fn log_payload() -> Vec<u8> {
	hex!(
		"
		f89b94c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2f863a0ddf252ad1be2c89b69c2b068fc37
		8daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000e9c1281aae66801fa3
		5ec404d5f2aea393ff6988a00000000000000000000000007a250d5630b4cf539739df2c5dacb4c6
		59f2488da000000000000000000000000000000000000000000000000003e973b5a5d1078e
	"
	)
	.to_vec()
}

pub fn new_tester<T: crate::Config>() -> sp_io::TestExternalities {
	new_tester_with_config::<T>(crate::GenesisConfig {
		initial_header: genesis_ethereum_header(),
		initial_difficulty: 0.into(),
	})
}

pub fn new_tester_with_config<T: crate::Config>(
	config: crate::GenesisConfig,
) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<T>().unwrap();

	GenesisBuild::<T>::assimilate_storage(&config, &mut storage).unwrap();

	let ext: sp_io::TestExternalities = storage.into();
	//ext.execute_with(|| <frame_system::Module<T>>::set_block_number();
	ext
}
