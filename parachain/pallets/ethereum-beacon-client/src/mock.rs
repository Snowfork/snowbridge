use super::*;
use crate as ethereum_beacon_client;
use frame_support::parameter_types;
use frame_system as system;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	AttesterSlashing, BeaconHeader, Body, SyncCommittee,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use std::{fs::File, path::PathBuf};

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
		EthereumBeaconClient: ethereum_beacon_client::{Pallet, Call, Config<T>, Storage, Event<T>},
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxSyncCommitteeSize: u32 = config::SYNC_COMMITTEE_SIZE as u32;
    pub const MaxProofBranchSize: u32 = 6;
    pub const MaxExtraDataSize: u32 = config::MAX_EXTRA_DATA_BYTES as u32;
    pub const MaxLogsBloomSize: u32 = config::MAX_LOGS_BLOOM_SIZE as u32;
    pub const MaxFeeRecipientSize: u32 = config::MAX_FEE_RECIPIENT_SIZE as u32;
    pub const MaxDepositDataSize: u32 = config::MAX_DEPOSITS as u32;
    pub const MaxPublicKeySize: u32 = config::PUBKEY_SIZE as u32;
    pub const MaxSignatureSize: u32 = config::SIGNATURE_SIZE as u32;
    pub const MaxProposerSlashingSize: u32 = config::MAX_PROPOSER_SLASHINGS as u32;
    pub const MaxAttesterSlashingSize: u32 = config::MAX_ATTESTER_SLASHINGS as u32;
    pub const MaxVoluntaryExitSize: u32 = config::MAX_VOLUNTARY_EXITS as u32;
    pub const MaxAttestationSize: u32 = config::MAX_ATTESTATIONS as u32;
    pub const MaxValidatorsPerCommittee: u32 = config::MAX_VALIDATORS_PER_COMMITTEE as u32;
}

impl ethereum_beacon_client::Config for Test {
    type Event = Event;
    type MaxSyncCommitteeSize = MaxSyncCommitteeSize;
    type MaxProofBranchSize = MaxProofBranchSize;
    type MaxExtraDataSize = MaxExtraDataSize;
    type MaxLogsBloomSize = MaxLogsBloomSize;
    type MaxFeeRecipientSize = MaxFeeRecipientSize;
    type MaxDepositDataSize = MaxDepositDataSize;
    type MaxPublicKeySize = MaxPublicKeySize;
    type MaxSignatureSize = MaxSignatureSize;
    type MaxProposerSlashingSize = MaxProposerSlashingSize;
    type MaxAttesterSlashingSize = MaxAttesterSlashingSize;
    type MaxVoluntaryExitSize = MaxVoluntaryExitSize;
    type MaxAttestationSize = MaxAttestationSize;
    type MaxValidatorsPerCommittee = MaxValidatorsPerCommittee;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_tester() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub struct SyncCommitteeTest {
	pub sync_committee: SyncCommittee<MaxSyncCommitteeSize>,
	pub result: H256,
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockBodyTest {
	pub body: Body<
	MaxFeeRecipientSize, 
	MaxLogsBloomSize, 
	MaxExtraDataSize, 
	MaxDepositDataSize, 
	MaxPublicKeySize, 
	MaxSignatureSize, 
	MaxProofBranchSize, 
	MaxProposerSlashingSize, 
	MaxAttesterSlashingSize, 
	MaxVoluntaryExitSize,
	MaxAttestationSize,
	MaxValidatorsPerCommittee,
	MaxSyncCommitteeSize>,
	pub result: H256,
}

pub struct BLSSignatureVerifyTest {
	pub sync_committee_bits: Vec<u8>,
	pub sync_committee_signature: Vec<u8>,
	pub pubkeys: Vec<PublicKey>,
	pub fork_version: ForkVersion,
	pub header: BeaconHeader,
	pub validators_root: H256,
}

fn fixture_path(name: &str) -> PathBuf {
	[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", name].iter().collect()
}

fn initial_sync_from_file(name: &str) -> InitialSync<MaxSyncCommitteeSize, MaxProofBranchSize> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn sync_committee_update_from_file(name: &str) -> SyncCommitteePeriodUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn finalized_header_update_from_file(name: &str) -> FinalizedHeaderUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn block_update_from_file(name: &str) -> BlockUpdate<
	MaxFeeRecipientSize, 
	MaxLogsBloomSize, 
	MaxExtraDataSize, 
	MaxDepositDataSize, 
	MaxPublicKeySize, 
	MaxSignatureSize, 
	MaxProofBranchSize, 
	MaxProposerSlashingSize, 
	MaxAttesterSlashingSize, 
	MaxVoluntaryExitSize,
	MaxAttestationSize,
	MaxValidatorsPerCommittee,
	MaxSyncCommitteeSize> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn attester_slashing_from_file(name: &str) -> AttesterSlashing<MaxValidatorsPerCommittee, MaxSignatureSize> {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn add_file_prefix(name: &str) -> String {
	let prefix = match config::IS_MINIMAL {
		true => "minimal_",
		false => "goerli_",
	};

	let mut result = prefix.to_owned();
	result.push_str(name);
	result
}

pub fn get_initial_sync() -> InitialSync<MaxSyncCommitteeSize, MaxProofBranchSize> {
	initial_sync_from_file(&add_file_prefix("initial_sync.json"))
}

pub fn get_committee_sync_period_update() -> SyncCommitteePeriodUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	sync_committee_update_from_file(&add_file_prefix("sync_committee_update.json"))
}

pub fn get_header_update() -> BlockUpdate<
	MaxFeeRecipientSize, 
	MaxLogsBloomSize, 
	MaxExtraDataSize, 
	MaxDepositDataSize, 
	MaxPublicKeySize, 
	MaxSignatureSize, 
	MaxProofBranchSize, 
	MaxProposerSlashingSize, 
	MaxAttesterSlashingSize, 
	MaxVoluntaryExitSize,
	MaxAttestationSize,
	MaxValidatorsPerCommittee,
	MaxSyncCommitteeSize> {
	block_update_from_file(&add_file_prefix("block_update.json"))
}

pub fn get_finalized_header_update() -> FinalizedHeaderUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	finalized_header_update_from_file(&add_file_prefix("finalized_header_update.json"))
}

pub fn get_validators_root() -> H256 {
	get_initial_sync().validators_root
}

pub fn get_sync_committee_test_data() -> SyncCommitteeTest {
	let sync_committee = get_committee_sync_period_update().next_sync_committee;
	let result: H256 = match config::IS_MINIMAL {
		true => hex!("92df9cdb8a742500dbf7afd3a7cce35805f818a3acbee8a26b7d6beff7d2c554").into(),
		false => hex!("c1bcfd9c44c8b9fec443530f7cf06f281c6b5d2d1ede77a486eea591fe79b0b5").into(),
	};

	SyncCommitteeTest { sync_committee, result }
}

pub fn get_block_body_test_data() -> BlockBodyTest {
	let update = get_header_update();

	BlockBodyTest { body: update.block.body, result: update.block_body_root }
}

pub fn get_bls_signature_verify_test_data() -> BLSSignatureVerifyTest {
	let finalized_update = get_finalized_header_update();
	let initial_sync = get_initial_sync();

	BLSSignatureVerifyTest {
		sync_committee_bits: finalized_update.sync_aggregate.sync_committee_bits.try_into().expect("sync committee bits are too long"),
		sync_committee_signature: finalized_update.sync_aggregate.sync_committee_signature.to_vec().try_into().expect("signature is too long"),
		pubkeys: initial_sync.current_sync_committee.pubkeys.to_vec().try_into().expect("pubkeys are too long"),
		fork_version: finalized_update.fork_version,
		header: finalized_update.attested_header,
		validators_root: initial_sync.validators_root,
	}
}

pub fn get_attester_slashing() -> AttesterSlashing<MaxValidatorsPerCommittee, MaxSignatureSize> {
	attester_slashing_from_file("attester_slashing.json")
}