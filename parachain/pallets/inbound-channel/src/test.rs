use super::*;

use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{Everything, GenesisBuild},
};
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use sp_std::convert::From;

use snowbridge_core::{Message, MessageDispatch, Proof};
use snowbridge_ethereum::{Header as EthereumHeader, Log, U256};

use hex_literal::hex;

use crate::{
	inbound as basic_inbound_channel,
	inbound::{envelope::Envelope, Error},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		BasicInboundChannel: basic_inbound_channel::{Pallet, Call, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
// Mock verifier
pub struct MockVerifier;

impl Verifier for MockVerifier {
	fn verify(message: &Message) -> Result<(Log, u64), DispatchError> {
		let log: Log = rlp::decode(&message.data).unwrap();
		Ok((log, 0))
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
	fn successful_dispatch_event(
		_: MessageId,
	) -> Option<<Test as frame_system::Config>::RuntimeEvent> {
		None
	}
}

impl basic_inbound_channel::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Verifier = MockVerifier;
	type MessageDispatch = MockMessageDispatch;
	type WeightInfo = ();
}

pub fn new_tester(source_channel: H160) -> sp_io::TestExternalities {
	new_tester_with_config(basic_inbound_channel::GenesisConfig { source_channel })
}

pub fn new_tester_with_config(
	config: basic_inbound_channel::GenesisConfig,
) -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn parse_origin(message: Message) -> H160 {
	let (log, _) = MockVerifier::verify(&message)
		.map_err(|err| {
			println!("mock verify: {:?}", err);
			err
		})
		.unwrap();
	let envelope = Envelope::try_from(log)
		.map_err(|err| {
			println!("envelope: {:?}", err);
			err
		})
		.unwrap();
	envelope.account
}

// The originating channel address for the messages below
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["86d9ac0bab011917f57b9e9607833b4340f9d4f8"];

// Ethereum Log:
//   address: 0xe4ab635d0bdc5668b3fcb4eaee1dec587998f4af (outbound channel contract)
//   topics: ...
//   data:
//     source: 0x8f5acf5f15d4c3d654a759b96bb674a236c8c0f3  (ETH bank contract)
//     nonce: 1
//     payload ...
const MESSAGE_DATA_0: [u8; 251] = hex!(
	"
	f8f99486d9ac0bab011917f57b9e9607833b4340f9d4f8e1a0daab80e8986999
	7d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb8c00000000000
	0000000000000089b4ab1ef20763630df9743acf155865600daff20000000000
	0000000000000004e00e6d2e9ea1e2af553de02a5172120bfa5c3e0000000000
	0000000000000000000000000000000000000000000000000000010000000000
	0000000000000000000000000000000000000000000000000000800000000000
	0000000000000000000000000000000000000000000000000000206172626974
	726172792d7061796c6f6164000000000000000000000000000000
"
);

// Ethereum Log:
//   address: 0xe4ab635d0bdc5668b3fcb4eaee1dec587998f4af (outbound channel contract)
//   topics: ...
//   data:
//     source: 0x8f5acf5f15d4c3d654a759b96bb674a236c8c0f3  (ETH bank contract)
//     nonce: 1
//     payload ...
const MESSAGE_DATA_1: [u8; 251] = hex!(
	"
	f8f99486d9ac0bab011917f57b9e9607833b4340f9d4f8e1a0daab80e8986999
	7d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb8c00000000000
	0000000000000089b4ab1ef20763630df9743acf155865600daff20000000000
	0000000000000004e00e6d2e9ea1e2af553de02a5172120bfa5c3e0000000000
	0000000000000000000000000000000000000000000000000000020000000000
	0000000000000000000000000000000000000000000000000000800000000000
	0000000000000000000000000000000000000000000000000000206172626974
	726172792d7061796c6f6164000000000000000000000000000000
"
);

#[test]
fn test_submit_with_invalid_source_channel() {
	new_tester(H160::zero()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
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
		let origin = RuntimeOrigin::signed(relayer);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message_1.clone()));

		let event_origin = parse_origin(message_1);
		let nonce: u64 = <Nonce<Test>>::get(event_origin.clone());
		assert_eq!(nonce, 1);

		// Submit message 2
		let message_2 = Message {
			data: MESSAGE_DATA_1.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message_2.clone()));

		let event_origin_2 = parse_origin(message_2);
		let nonce: u64 = <Nonce<Test>>::get(event_origin_2.clone());
		assert_eq!(nonce, 2);
	});
}

#[test]
fn test_submit_with_invalid_nonce() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(BasicInboundChannel::submit(origin.clone(), message.clone()));

		let event_origin = parse_origin(message.clone());
		let nonce: u64 = <Nonce<Test>>::get(event_origin);
		assert_eq!(nonce, 1);

		// Submit the same again
		assert_noop!(
			BasicInboundChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidNonce
		);
	});
}
