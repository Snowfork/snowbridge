// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{ConstU128, Everything},
	weights::IdentityFee,
};
use hex_literal::hex;
use snowbridge_beacon_primitives::{Fork, ForkVersions};
use snowbridge_core::{
	inbound::{Message, Proof, VerificationError},
	ParaId,
};
use snowbridge_router_primitives::inbound::MessageToXcm;
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, DispatchError, MultiSignature, TokenError,
};
use sp_std::convert::From;
use xcm::v3::{prelude::*, MultiAssets, SendXcm};

use crate::{self as inbound_queue, Error, Event as InboundQueueEvent};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
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

type Balance = u128;

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = u64;
	type Block = Block;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
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
	fn verify(_: &Message) -> Result<(), VerificationError> {
		Ok(())
	}
}

const GATEWAY_ADDRESS: [u8; 20] = hex!["eda338e4dc46038493b885327842fd3e301cab39"];

parameter_types! {
	pub const EthereumNetwork: xcm::v3::NetworkId = xcm::v3::NetworkId::Ethereum { chain_id: 15 };
	pub const GatewayAddress: H160 = H160(GATEWAY_ADDRESS);
	pub const CreateAssetCall: [u8;2] = [53, 0];
	pub const CreateAssetExecutionFee: u128 = 2_000_000_000;
	pub const CreateAssetDeposit: u128 = 100_000_000_000;
	pub const SendTokenExecutionFee: u128 = 1_000_000_000;
	pub const InitialFund: u128 = 1_000_000_000_000;
	pub const TreasuryAccount: [u8; 32] = [0; 32];
}

#[cfg(feature = "runtime-benchmarks")]
impl<T: snowbridge_ethereum_beacon_client::Config> BenchmarkHelper<T> for Test {
	// not implemented since the MockVerifier is used for tests
	fn initialize_storage(_: H256, _: CompactExecutionHeader) {}
}

// Mock XCM sender that always succeeds
pub struct MockXcmSender;

impl SendXcm for MockXcmSender {
	type Ticket = Xcm<()>;

	fn validate(
		dest: &mut Option<MultiLocation>,
		xcm: &mut Option<xcm::v3::Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		match dest {
			Some(MultiLocation { interior, .. }) => {
				if let X1(Parachain(1001)) = interior {
					return Err(XcmpSendError::NotApplicable)
				}
				Ok((xcm.clone().unwrap(), MultiAssets::default()))
			},
			_ => Ok((xcm.clone().unwrap(), MultiAssets::default())),
		}
	}

	fn deliver(xcm: Self::Ticket) -> core::result::Result<XcmHash, XcmpSendError> {
		let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
		Ok(hash)
	}
}

impl inbound_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Verifier = MockVerifier;
	type Token = Balances;
	type Reward = ConstU128<100>;
	type XcmSender = MockXcmSender;
	type WeightInfo = ();
	type GatewayAddress = GatewayAddress;
	type MessageConverter = MessageToXcm<
		CreateAssetCall,
		CreateAssetExecutionFee,
		CreateAssetDeposit,
		SendTokenExecutionFee,
		TreasuryAccount,
		AccountId,
		Balance,
	>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = Test;
	type WeightToFee = IdentityFee<u128>;
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

fn setup() {
	System::set_block_number(1);
	Balances::mint_into(
		&sibling_sovereign_account::<Test>(ASSET_HUB_PARAID.into()),
		InitialFund::get(),
	)
	.unwrap();
	Balances::mint_into(
		&sibling_sovereign_account::<Test>(TEMPLATE_PARAID.into()),
		InitialFund::get(),
	)
	.unwrap();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| setup());
	ext
}

// dest para is 1000
const OUTBOUND_QUEUE_EVENT_LOG: [u8; 254] = hex!(
	"
	f8fc94eda338e4dc46038493b885327842fd3e301cab39f863a05066fbba677e15936860e04088ca4cad3acd4c19706962196a5346f1457f7169a000000000000000000000000000000000000000000000000000000000000003e8a0afad3c9777134532ae230b4fad334eef2e0dacbb965920412a7eaa59b07d640fb88000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001e000f000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000
	"
);

// dest para is 1001
const OUTBOUND_QUEUE_EVENT_LOG_INVALID_DEST: [u8; 254] = hex!(
	"
	f8fc94eda338e4dc46038493b885327842fd3e301cab39f863a05066fbba677e15936860e04088ca4cad3acd4c19706962196a5346f1457f7169a000000000000000000000000000000000000000000000000000000000000003e9a0afad3c9777134532ae230b4fad334eef2e0dacbb965920412a7eaa59b07d640fb88000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001e000f000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000
	"
);

// gateway in message does not match configured gateway in runtimeå
const BAD_OUTBOUND_QUEUE_EVENT_LOG: [u8; 254] = hex!(
	"
	f8fc940000000000000000000000000000000000000000f863a05066fbba677e15936860e04088ca4cad3acd4c19706962196a5346f1457f7169a000000000000000000000000000000000000000000000000000000000000003e8a0afad3c9777134532ae230b4fad334eef2e0dacbb965920412a7eaa59b07d640fb88000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001e000f000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000
	"
);

const XCM_HASH: [u8; 32] = [
	128, 179, 214, 84, 162, 117, 153, 35, 165, 19, 240, 82, 52, 249, 101, 207, 236, 104, 231, 147,
	109, 81, 111, 125, 133, 72, 179, 4, 82, 206, 77, 149,
];
const ASSET_HUB_PARAID: u32 = 1000u32;
const TEMPLATE_PARAID: u32 = 1001u32;

#[test]
fn test_submit_happy_path() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);

		// Deposit funds into sovereign account of Asset Hub (Statemint)
		let sovereign_account = sibling_sovereign_account::<Test>(ASSET_HUB_PARAID.into());
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
			dest: ASSET_HUB_PARAID.into(),
			nonce: 1,
			message_id: XCM_HASH,
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
		let sovereign_account = sibling_sovereign_account::<Test>(1001u32.into());
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
		let sovereign_account = sibling_sovereign_account::<Test>(ASSET_HUB_PARAID.into());
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
		let sovereign_account = sibling_sovereign_account::<Test>(ASSET_HUB_PARAID.into());
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

		let nonce: u64 = <Nonce<Test>>::get(ParaId::from(1000));
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

		// Reset balance of sovereign_account to zero so to trigger the FundsUnavailable error
		let sovereign_account = sibling_sovereign_account::<Test>(ASSET_HUB_PARAID.into());
		Balances::set_balance(&sovereign_account, 0);

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
			TokenError::FundsUnavailable
		);
	});
}

#[test]
fn test_set_operating_mode() {
	new_tester().execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = RuntimeOrigin::signed(relayer);
		let message = Message {
			data: OUTBOUND_QUEUE_EVENT_LOG.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};

		assert_ok!(InboundQueue::set_operating_mode(
			RuntimeOrigin::root(),
			snowbridge_core::BasicOperatingMode::Halted
		));

		assert_noop!(InboundQueue::submit(origin, message), Error::<Test>::Halted);
	});
}

#[test]
fn test_set_operating_mode_root_only() {
	new_tester().execute_with(|| {
		assert_noop!(
			InboundQueue::set_operating_mode(
				RuntimeOrigin::signed(Keyring::Bob.into()),
				snowbridge_core::BasicOperatingMode::Halted
			),
			DispatchError::BadOrigin
		);
	});
}
