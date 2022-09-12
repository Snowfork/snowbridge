use super::*;
use crate as ethereum_beacon_client;
use frame_support::{BoundedVec, parameter_types};
use frame_system as system;
use hex_literal::hex;
use snowbridge_beacon_primitives::{
	AttesterSlashing, BeaconHeader, Body, SyncCommittee, SyncAggregate, SyncCommitteePeriodUpdateSerialize, FinalizedHeaderUpdateSerialize, BlockUpdateSerialize, AttesterSlashingSerialize, Eth1Data, IndexedAttestation, AttestationData,
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
		EthereumBeaconClient: ethereum_beacon_client::{Pallet, Call, Config, Storage, Event<T>},
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
	pub const MaxSyncCommitteeSize: u32 = 512;
	pub const MaxProofBranchSize: u32 = 10;
	pub const MaxExtraDataSize: u32 = 32;
	pub const MaxLogsBloomSize: u32 = 256;
	pub const MaxFeeRecipientSize: u32 = 20;
	pub const MaxDepositDataSize: u32 = 16;
	pub const MaxPublicKeySize: u32 = 48;
	pub const MaxSignatureSize: u32 = 96;
	pub const MaxProposerSlashingSize: u32 = 16;
	pub const MaxAttesterSlashingSize: u32 = 2;
	pub const MaxVoluntaryExitSize: u32 = 16;
	pub const MaxAttestationSize: u32 = 128;
	pub const MaxValidatorsPerCommittee: u32 = 2048;
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

fn initial_sync_from_file(name: &str) -> InitialSyncSerialize {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn sync_committee_update_from_file(name: &str) -> SyncCommitteePeriodUpdateSerialize{
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn finalized_header_update_from_file(name: &str) -> FinalizedHeaderUpdateSerialize {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn block_update_from_file(name: &str) -> BlockUpdateSerialize {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn attester_slashing_from_file(name: &str) -> AttesterSlashingSerialize {
	let filepath = fixture_path(name);
	serde_json::from_reader(File::open(&filepath).unwrap()).unwrap()
}

fn add_file_prefix(name: &str) -> String {
	let prefix = match config::IS_MINIMAL {
		true => "minimal_",
		false => "ropsten_",
	};

	let mut result = prefix.to_owned();
	result.push_str(name);
	result
}

pub fn get_initial_sync() -> InitialSync<MaxSyncCommitteeSize, MaxProofBranchSize> {
	let initial_sync = initial_sync_from_file(&add_file_prefix("initial_sync.json"));

	let bounded_branch: BoundedVec<H256, MaxProofBranchSize> =
		initial_sync.current_sync_committee_branch.clone().try_into().expect("proof branch is too long");

	let bounded_sync_committee_pubkeys: BoundedVec<PublicKey, MaxSyncCommitteeSize> = 
		initial_sync.current_sync_committee.pubkeys.clone().try_into().expect("pubkeys are too many");

	let bounded_sync_committee = SyncCommittee{
		pubkeys: bounded_sync_committee_pubkeys,
		aggregate_pubkey: initial_sync.current_sync_committee.aggregate_pubkey.clone()
	};

	InitialSync{ 
		header: initial_sync.header.clone(), 
		current_sync_committee: bounded_sync_committee, 
		current_sync_committee_branch: bounded_branch, 
		validators_root: initial_sync.validators_root,
	}
}

pub fn get_committee_sync_period_update() -> SyncCommitteePeriodUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	let update = sync_committee_update_from_file(&add_file_prefix("sync_committee_update.json"));

	let bounded_sync_committee_branch: BoundedVec<H256, MaxProofBranchSize> =
		update.next_sync_committee_branch.clone().try_into().expect("sync committee proof branch is too long");

	let bounded_finality_branch: BoundedVec<H256, MaxProofBranchSize> =
		update.finality_branch.clone().try_into().expect("finalized header proof branch is too long");

	let bounded_sync_committee_pubkeys: BoundedVec<PublicKey, MaxSyncCommitteeSize> = 
		update.next_sync_committee.pubkeys.clone().try_into().expect("pubkeys are too many");

	let bounded_sync_committee = SyncCommittee{
		pubkeys: bounded_sync_committee_pubkeys,
		aggregate_pubkey: update.next_sync_committee.aggregate_pubkey.clone()
	};

	let sync_committee_bits: BoundedVec<u8, MaxSyncCommitteeSize> =
		update.sync_aggregate.sync_committee_bits.clone().try_into().expect("sync committee bits are too long");

	let sync_committee_signature: BoundedVec<u8, MaxSignatureSize> =
		update.sync_aggregate.sync_committee_signature.clone().try_into().expect("sync committee sinature is too long");

	let sync_aggregate = SyncAggregate{
		sync_committee_bits: sync_committee_bits,
		sync_committee_signature: sync_committee_signature,
	};

	SyncCommitteePeriodUpdate{ 
		attested_header: update.attested_header.clone(), 
		next_sync_committee: bounded_sync_committee, 
		next_sync_committee_branch: bounded_sync_committee_branch, 
		finalized_header: update.finalized_header.clone(), 
		finality_branch: bounded_finality_branch, 
		sync_aggregate: sync_aggregate, 
		fork_version: update.fork_version.clone(), 
		sync_committee_period: update.sync_committee_period
	}
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
	let update = block_update_from_file(&add_file_prefix("block_update.json"));

	let attester_slashings = Vec::new();

	for attester_slashing in update.block.body.attester_slashings.iter() {
        attester_slashings.push(AttesterSlashing{ 
			attestation_1: IndexedAttestation{
				attesting_indices: attester_slashing.attestation_1.attesting_indices.clone().try_into().expect("attesting indices is too long"),
				data: AttestationData{
					slot: attester_slashing.attestation_1.data.slot,
					index: attester_slashing.attestation_1.data.index,
					beacon_block_root: attester_slashing.attestation_1.data.beacon_block_root,
					source: todo!(),
					target: todo!(),
				},
				signature: attester_slashing.attestation_1.signature.clone().try_into().expect("signature is too long"),
			}, 
			attestation_2: todo!() 
		});
    }

	BlockUpdate { 
		block: snowbridge_beacon_primitives::BeaconBlock { 
			slot: update.block.slot, 
			proposer_index: update.block.proposer_index, 
			parent_root: update.block.parent_root, 
			state_root: update.block.state_root, 
			body: Body{ 
				randao_reveal: update.block.body.randao_reveal.clone().try_into().expect("randao reveal is too long"), 
				eth1_data: Eth1Data { 
					deposit_root: update.block.body.eth1_data.deposit_root, 
					deposit_count: update.block.body.eth1_data.deposit_count, 
					block_hash: update.block.body.eth1_data.block_hash, 
				}, 
				graffiti: update.block.body.graffiti, 
				proposer_slashings: update.block.body.proposer_slashings.clone().try_into().expect("too many proposer slashings"), 
				attester_slashings: update.block.body.attester_slashings.clone().try_into().expect("too many attester slashings"), 
				attestations: update.block.body.attester_slashings.clone().try_into().expect("too many attestations"), 
				deposits: todo!(), 
				voluntary_exits: todo!(), 
				sync_aggregate: SyncAggregate{
					sync_committee_bits: update.block.body.sync_aggregate.sync_committee_bits.clone().try_into().expect("sync committee bits are too long"),
					sync_committee_signature: update.block.body.sync_aggregate.sync_committee_signature.clone().try_into().expect("sync committee sinature is too long"),
				}, 
				execution_payload: todo!() 
			}
		}, 
		block_body_root: update.block_body_root, 
		sync_aggregate: SyncAggregate{
			sync_committee_bits: update.sync_aggregate.sync_committee_bits.clone().try_into().expect("sync committee bits are too long"),
			sync_committee_signature: update.sync_aggregate.sync_committee_signature.clone().try_into().expect("sync committee sinature is too long"),
		}, 
		fork_version: update.fork_version
	}
}

pub fn get_finalized_header_update() -> FinalizedHeaderUpdate<MaxSignatureSize, MaxProofBranchSize, MaxSyncCommitteeSize> {
	finalized_header_update_from_file(&add_file_prefix("finalized_header_update.json"))
}

pub fn get_validators_root() -> H256 {
	get_initial_sync().validators_root
}

pub fn get_current_sync_committee_for_current_committee_update() -> SyncCommittee<MaxSyncCommitteeSize> {
	let initial_sync = initial_sync_from_file(&add_file_prefix("initial_sync.json"));

	let bounded_sync_committee_pubkeys: BoundedVec<PublicKey, MaxSyncCommitteeSize> = 
		initial_sync.current_sync_committee.pubkeys.clone().try_into().expect("pubkeys are too many");

	SyncCommittee{
		pubkeys: bounded_sync_committee_pubkeys,
		aggregate_pubkey: initial_sync.current_sync_committee.aggregate_pubkey.clone()
	}
}

pub fn get_current_sync_committee_for_finalized_header_update() -> SyncCommittee<MaxSyncCommitteeSize> {
	let initial_sync = initial_sync_from_file(&add_file_prefix("initial_sync.json"));

	let bounded_sync_committee_pubkeys: BoundedVec<PublicKey, MaxSyncCommitteeSize> = 
		initial_sync.current_sync_committee.pubkeys.clone().try_into().expect("pubkeys are too many");

	SyncCommittee{
		pubkeys: bounded_sync_committee_pubkeys,
		aggregate_pubkey: initial_sync.current_sync_committee.aggregate_pubkey.clone()
	}
}

pub fn get_sync_committee_test_data() -> SyncCommitteeTest {
	let sync_committee = get_committee_sync_period_update().next_sync_committee;
	let result: H256 = match config::IS_MINIMAL {
		true => hex!("fc5afdee715774e88c160f1ef6b81dd0cd47f769fca7062a8881ab932a510e18").into(),
		false => hex!("b51b706921f2c94eff39fd6c3377b6acf6a050c077db87e3ee0a013023d75f82").into(),
	};

	SyncCommitteeTest { sync_committee, result }
}

pub fn get_block_body_test_data() -> BlockBodyTest {
	let body = get_header_update().block.body;
	let result: H256 = match config::IS_MINIMAL {
		true => hex!("90049ca395d637c1643af699f1aba29aa10d14e8b267fc92f71a87b421641d00").into(),
		false => hex!("c8b6dade675a2453c0d2702d66626b18bbb4ed9d00e542a7763ce9b6a406f47c").into(),
	};

	BlockBodyTest { body, result }
}

pub fn get_current_sync_committee_for_header_update() -> SyncCommittee<MaxSignatureSize> {
	get_initial_sync().current_sync_committee
}

pub fn get_bls_signature_verify_test_data() -> BLSSignatureVerifyTest {
	let finalized_update = get_finalized_header_update();
	let initial_sync = get_initial_sync();

	BLSSignatureVerifyTest {
		sync_committee_bits: finalized_update.sync_aggregate.sync_committee_bits,
		sync_committee_signature: finalized_update.sync_aggregate.sync_committee_signature,
		pubkeys: initial_sync.current_sync_committee.pubkeys,
		fork_version: finalized_update.fork_version,
		header: finalized_update.attested_header,
		validators_root: initial_sync.validators_root,
	}
}

pub fn get_attester_slashing() ->  AttesterSlashing<MaxValidatorsPerCommittee, MaxSignatureSize> {
	attester_slashing_from_file("attester_slashing.json")
}
