// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{ConstU64, Everything},
};
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	ArithmeticError, MultiSignature,
};
use sp_std::convert::From;

use snowbridge_beacon_primitives::{Fork, ForkVersions};
use snowbridge_core::inbound::{Message, Proof};
use snowbridge_ethereum::Log;

use hex_literal::hex;
use xcm::v3::{prelude::*, MultiAssets, SendXcm};

use crate::{self as inbound_queue, envelope::Envelope, Error, Event as InboundQueueEvent};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		EthereumBeaconClient: snowbridge_ethereum_beacon_client::{Pallet, Call, Storage, Event<T>},
		InboundQueue: inbound_queue::{Pallet, Call, Storage, Event<T>},
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

parameter_types! {
	pub const ExecutionHeadersPruneThreshold: u32 = 10;
	pub const ChainForkVersions: ForkVersions = ForkVersions{
		genesis: Fork {
			version: [0, 0, 0, 1], // 0x00000001
			epoch: 0,
		},
		altair: Fork {
			version: [1, 0, 0, 1], // 0x01000001
			epoch: 0,
		},
		bellatrix: Fork {
			version: [2, 0, 0, 1], // 0x02000001
			epoch: 0,
		},
		capella: Fork {
			version: [3, 0, 0, 1], // 0x03000001
			epoch: 0,
		},
	};
}

impl snowbridge_ethereum_beacon_client::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ForkVersions = ChainForkVersions;
	type MaxExecutionHeadersToKeep = ExecutionHeadersPruneThreshold;
	type WeightInfo = ();
}

// Mock verifier
pub struct MockVerifier;

impl Verifier for MockVerifier {
	fn verify(message: &Message) -> Result<Log, DispatchError> {
		let log: Log = rlp::decode(&message.data).unwrap();
		Ok(log)
	}
}

const GATEWAY_ADDRESS: [u8; 20] = hex!["eda338e4dc46038493b885327842fd3e301cab39"];

parameter_types! {
	pub const EthereumNetwork: xcm::v3::NetworkId = xcm::v3::NetworkId::Ethereum { chain_id: 15 };
	pub const GatewayAddress: H160 = H160(GATEWAY_ADDRESS);
	pub const RegisterCallIndex: [u8;2] = [53, 0];
}

#[cfg(feature = "runtime-benchmarks")]
impl<T: snowbridge_ethereum_beacon_client::Config> BenchmarkHelper<T> for Test {
	// not implemented since the MockVerifier is used for tests
	fn initialize_storage(_: H256, _: CompactExecutionHeader) {}
}

// Mock XCM sender that always succeeds
pub struct MockXcmSender;

impl SendXcm for MockXcmSender {
	type Ticket = ();

	fn validate(
		dest: &mut Option<MultiLocation>,
		_: &mut Option<xcm::v3::Xcm<()>>,
	) -> xcm::v3::SendResult<Self::Ticket> {
		match dest {
			Some(MultiLocation { interior, .. }) => {
				if let X1(Parachain(1001)) = interior {
					return Err(XcmpSendError::NotApplicable)
				}
				Ok(((), MultiAssets::default()))
			},
			_ => Ok(((), MultiAssets::default())),
		}
	}

	fn deliver(_: Self::Ticket) -> core::result::Result<XcmHash, XcmpSendError> {
		Ok(H256::zero().into())
	}
}

impl inbound_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Verifier = MockVerifier;
	type Token = Balances;
	type Reward = ConstU64<100>;
	type XcmSender = MockXcmSender;
	type WeightInfo = ();
	type GatewayAddress = GatewayAddress;
	type MessageConverter =
		VersionedMessageToXcmConverter<RegisterCallIndex, ConstantFeeForInboundMessage>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = Test;
}

fn last_events(n: usize) -> Vec<RuntimeEvent> {
	frame_system::Pallet::<Test>::events()
		.into_iter()
		.rev()
		.take(n)
		.rev()
		.map(|e| e.event)
		.collect()
}

fn expect_events(e: Vec<RuntimeEvent>) {
	assert_eq!(last_events(e.len()), e);
}

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn parse_dest(message: Message) -> ParaId {
	let log = MockVerifier::verify(&message)
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
	envelope.dest
}

// dest para is 1000
const OUTBOUND_QUEUE_EVENT_LOG: [u8; 253] = hex!(
	"
	f8fb94eda338e4dc46038493b885327842fd3e301cab39f842a0d56f1b8dfd3ba41f19c499ceec5f9546f61befa5f10398a75d7dba53a219fecea000000000000000000000000000000000000000000000000000000000000003e8b8a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000032000f0000000000000000eda338e4dc46038493b885327842fd3e301cab3987d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000000000000000000000000000
	"
);

// dest para is 1001
const OUTBOUND_QUEUE_EVENT_LOG_INVALID_DEST: [u8; 253] = hex!(
	"
	f8fb94eda338e4dc46038493b885327842fd3e301cab39f842a0d56f1b8dfd3ba41f19c499ceec5f9546f61befa5f10398a75d7dba53a219fecea000000000000000000000000000000000000000000000000000000000000003e9b8a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000032000f0000000000000000eda338e4dc46038493b885327842fd3e301cab3987d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000000000000000000000000000
	"
);

// gateway in message does not match configured gateway in runtime
const BAD_OUTBOUND_QUEUE_EVENT_LOG: [u8; 253] = hex!(
	"
	f8fb940000000000000000000000000000000000000000f842a0d56f1b8dfd3ba41f19c499ceec5f9546f61befa5f10398a75d7dba53a219fecea000000000000000000000000000000000000000000000000000000000000003e9b8a0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000032000f0000000000000000eda338e4dc46038493b885327842fd3e301cab3987d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000000000000000000000000000
	"
);

use snowbridge_core::ParaId;
use snowbridge_router_primitives::inbound::{
	ConstantFeeForInboundMessage, VersionedMessageToXcmConverter,
};

#[test]
fn test_submit_happy_path() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Deposit funds into sovereign account of Asset Hub (Statemint)
		let dest_para: ParaId = 1000u32.into();
		let sovereign_account: AccountId = dest_para.into_account_truncating();
		println!("account: {}", sovereign_account);
		let _ = Balances::mint_into(&sovereign_account, 10000);

		// Submit message
		let message = Message {
			data: OUTBOUND_QUEUE_EVENT_LOG.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(InboundQueue::submit(origin.clone(), message.clone()));
		expect_events(vec![InboundQueueEvent::MessageReceived {
			dest: dest_para,
			nonce: 1,
			xcm_hash: H256::zero().into(),
		}
		.into()]);
	});
}

#[test]
fn test_submit_xcm_send_failure() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Deposit funds into sovereign account of parachain 1001
		let dest_para: ParaId = 1001u32.into();
		let sovereign_account: AccountId = dest_para.into_account_truncating();
		println!("account: {}", sovereign_account);
		let _ = Balances::mint_into(&sovereign_account, 10000);

		// Submit message
		let message = Message {
			data: OUTBOUND_QUEUE_EVENT_LOG_INVALID_DEST.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_noop!(
			InboundQueue::submit(origin.clone(), message.clone()),
			Error::<Test>::Send(crate::SendError::NotApplicable)
		);
	});
}

#[test]
fn test_submit_with_invalid_gateway() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Deposit funds into sovereign account of Asset Hub (Statemint)
		let dest_para: ParaId = 1000u32.into();
		let sovereign_account: AccountId = dest_para.into_account_truncating();
		let _ = Balances::mint_into(&sovereign_account, 10000);

		// Submit message
		let message = Message {
			data: BAD_OUTBOUND_QUEUE_EVENT_LOG.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_noop!(
			InboundQueue::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidGateway
		);
	});
}

#[test]
fn test_submit_with_invalid_nonce() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Deposit funds into sovereign account of Asset Hub (Statemint)
		let dest_para: ParaId = 1000u32.into();
		let sovereign_account: AccountId = dest_para.into_account_truncating();
		let _ = Balances::mint_into(&sovereign_account, 10000);

		// Submit message
		let message = Message {
			data: OUTBOUND_QUEUE_EVENT_LOG.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(InboundQueue::submit(origin.clone(), message.clone()));

		let event_dest = parse_dest(message.clone());
		let nonce: u64 = <Nonce<Test>>::get(event_dest);
		assert_eq!(nonce, 1);

		// Submit the same again
		assert_noop!(
			InboundQueue::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn test_submit_no_funds_to_reward_relayers() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Create sovereign account for Asset Hub (Statemint), but with no funds to cover rewards
		let dest_para: ParaId = 1000u32.into();
		let sovereign_account: AccountId = dest_para.into_account_truncating();
		assert_ok!(Balances::mint_into(&sovereign_account, 2));

		// Submit message
		let message = Message {
			data: OUTBOUND_QUEUE_EVENT_LOG.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_noop!(
			InboundQueue::submit(origin.clone(), message.clone()),
			// should actually be `NoFunds`. See this bug in substrate:
			// https://github.com/paritytech/substrate/issues/13866
			ArithmeticError::Underflow
		);
	});
}
