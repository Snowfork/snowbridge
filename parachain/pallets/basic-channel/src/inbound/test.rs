use super::*;

use sp_core::{H160, H256};
use frame_support::{
	assert_ok, assert_noop,
	parameter_types,
	dispatch::DispatchError
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, IdentifyAccount, Verify}, testing::Header, MultiSignature
};
use sp_keyring::AccountKeyring as Keyring;
use sp_std::convert::From;

use artemis_core::{MessageDispatch, Message, Proof};
use artemis_ethereum::{Header as EthereumHeader, Log, U256};

use hex_literal::hex;

use crate::inbound::Error;

use crate::inbound as basic_inbound_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		BasicInboundChannel: basic_inbound_channel::{Module, Call, Storage, Event},
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
// Mock verifier
pub struct MockVerifier;

impl Verifier for MockVerifier {
	fn verify(message: &Message) -> Result<Log, DispatchError> {
		let log: Log = rlp::decode(&message.data).unwrap();
		Ok(log)
	}

	fn initialize_storage(_: Vec<EthereumHeader>, _: U256, _: u8) -> Result<(), &'static str> {
		Ok(())
	}
}

// Mock Dispatch
pub struct MockMessageDispatch;

impl MessageDispatch<Test, MessageId> for MockMessageDispatch {
	fn dispatch(_: H160, _: MessageId, _: &[u8]) {}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_dispatch_event(_: MessageId) -> Option<<Test as system::Config>::Event> {
		None
	}
}

impl basic_inbound_channel::Config for Test {
	type Event = Event;
	type Verifier = MockVerifier;
	type MessageDispatch = MockMessageDispatch;
}

pub fn new_tester(source_channel: H160) -> sp_io::TestExternalities {
	new_tester_with_config(basic_inbound_channel::GenesisConfig {
		source_channel,
	})
}

pub fn new_tester_with_config(config: basic_inbound_channel::GenesisConfig) -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	config.assimilate_storage(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}


// The originating channel address for the messages below
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["2ffa5ecdbe006d30397c7636d3e015eee251369f"];

const MESSAGE_DATA_0: [u8; 317] = hex!(
	"
	f9013a942ffa5ecdbe006d30397c7636d3e015eee251369fe1a0daab80e898699
	97d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb9010000000000
	000000000000000089b4ab1ef20763630df9743acf155865600daff2000000000
	000000000000000774667629726ec1fabebcec0d9139bd1c8f72a230000000000
	00000000000000000000000000000000000000000000000000000100000000000
	00000000000000000000000000000000000000000000000000080000000000000
	00000000000000000000000000000000000000000000000000570c0189b4ab1ef
	20763630df9743acf155865600daff200d43593c715fdd31c61141abd04a99fd6
	822c8558854ccde39a5684e7a56da27d0000c16ff286230000000000000000000
	0000000000000000000000000000000000000000000000000
"
);

const MESSAGE_DATA_1: [u8; 317] = hex!(
	"
	f9013a942ffa5ecdbe006d30397c7636d3e015eee251369fe1a0daab80e898699
	97d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb9010000000000
	000000000000000089b4ab1ef20763630df9743acf155865600daff2000000000
	000000000000000774667629726ec1fabebcec0d9139bd1c8f72a230000000000
	00000000000000000000000000000000000000000000000000000200000000000
	00000000000000000000000000000000000000000000000000080000000000000
	00000000000000000000000000000000000000000000000000570c0189b4ab1ef
	20763630df9743acf155865600daff200d43593c715fdd31c61141abd04a99fd6
	822c8558854ccde39a5684e7a56da27d0000c16ff286230000000000000000000
	0000000000000000000000000000000000000000000000000
"
);

#[test]
fn test_submit_with_invalid_source_channel() {
	new_tester(H160::zero()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default()
			},
		};
		assert_noop!(
			BasicInboundChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidSourceChannel
		);
	});
}

#[test]
fn test_submit() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);
		let eth_origin = H160::from_slice(&hex!("89b4ab1ef20763630df9743acf155865600daff2")[..]);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default()
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message_1));
		let nonce: u64 = Nonces::get(eth_origin);
		assert_eq!(nonce, 1);

		// Submit message 2
		let message_2 = Message {
			data: MESSAGE_DATA_1.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default()
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message_2));
		let nonce: u64 = Nonces::get(eth_origin);
		assert_eq!(nonce, 2);
	});
}

#[test]
fn test_submit_with_invalid_nonce() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);
		let eth_origin = H160::from_slice(&hex!("89b4ab1ef20763630df9743acf155865600daff2")[..]);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default()
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message.clone()));
		let nonce: u64 = Nonces::get(eth_origin);
		assert_eq!(nonce, 1);

		// Submit the same again
		assert_noop!(
			BasicInboundChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidNonce
		);
	});
}
