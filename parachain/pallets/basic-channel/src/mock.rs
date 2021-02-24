use super::*;

use frame_support::{dispatch::DispatchError, parameter_types};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use sp_std::convert::From;

use artemis_core::{ChannelId, MessageCommitment, MessageDispatch, MessageId};
use artemis_ethereum::Log;

use crate as basic_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		BasicChannel: basic_channel::{Module, Call, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
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
// Mock verifier that gives the green light to all messages
pub struct MockVerifier;

impl Verifier for MockVerifier {
	fn verify(message: &Message) -> Result<Log, DispatchError> {
		let log: Log = rlp::decode(&message.data).unwrap();
		Ok(log)
	}
}

pub struct MockMessageCommitment;

impl MessageCommitment for MockMessageCommitment {
	fn add(_channel_id: ChannelId, _target: H160, _nonce: u64, _payload: &[u8]) -> DispatchResult {
		Ok(())
	}
}

pub struct MockMessageDispatch;

impl MessageDispatch<MessageId> for MockMessageDispatch {
	fn dispatch(_: H160, _: MessageId, _: &[u8]) {}
}

impl Config for Test {
	type Event = Event;
	type Verifier = MockVerifier;
	type MessageCommitment = MockMessageCommitment;
	type MessageDispatch = MockMessageDispatch;
}

pub fn new_tester() -> sp_io::TestExternalities {
	new_tester_with_config(basic_channel::GenesisConfig {
		source_channel: H160::zero(),
	})
}

pub fn new_tester_with_source_channel(channel: H160) -> sp_io::TestExternalities {
	new_tester_with_config(basic_channel::GenesisConfig {
		source_channel: channel,
	})
}

pub fn new_tester_with_config(config: basic_channel::GenesisConfig) -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	config.assimilate_storage(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
