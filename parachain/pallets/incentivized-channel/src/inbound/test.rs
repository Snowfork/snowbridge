use super::*;

use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{Currency, Everything, GenesisBuild},
};
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Convert, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, Perbill,
};
use sp_std::{convert::From, marker::PhantomData};

use snowbridge_core::{Message, MessageDispatch, Proof};
use snowbridge_ethereum::{Header as EthereumHeader, Log, U256};

use hex_literal::hex;

use crate::inbound::Error;

use crate::inbound as incentivized_inbound_channel;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		IncentivizedInboundChannel: incentivized_inbound_channel::{Pallet, Call, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

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
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	/// The ubiquitous event type.
	type Event = Event;
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
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
	fn successful_dispatch_event(_: MessageId) -> Option<<Test as frame_system::Config>::Event> {
		None
	}
}

parameter_types! {
	pub SourceAccount: AccountId = Keyring::Eve.into();
	pub TreasuryAccount: AccountId = Keyring::Dave.into();
}

pub struct FeeConverter<T: Config>(PhantomData<T>);

impl<T: Config> Convert<U256, Option<BalanceOf<T>>> for FeeConverter<T> {
	fn convert(_: U256) -> Option<BalanceOf<T>> {
		Some(100u32.into())
	}
}

impl incentivized_inbound_channel::Config for Test {
	type Event = Event;
	type Verifier = MockVerifier;
	type MessageDispatch = MockMessageDispatch;
	type Currency = Balances;
	type SourceAccount = SourceAccount;
	type TreasuryAccount = TreasuryAccount;
	type FeeConverter = FeeConverter<Self>;
	type UpdateOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type WeightInfo = ();
}

pub fn new_tester(source_channel: H160) -> sp_io::TestExternalities {
	new_tester_with_config(incentivized_inbound_channel::GenesisConfig {
		source_channel,
		reward_fraction: Perbill::from_percent(80),
	})
}

pub fn new_tester_with_config(
	config: incentivized_inbound_channel::GenesisConfig,
) -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

// The originating channel address for the messages below
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["4130819912a398f4eb84e7f16ed443232ba638b5"];

// Message with nonce = 1
const MESSAGE_DATA_0: [u8; 317] = hex!(
	"
	f9013a944130819912a398f4eb84e7f16ed443232ba638b5e1a05e9ae1d7c484
	f74d554a503aa825e823725531d97e784dd9b1aacdb58d1f7076b90100000000
	000000000000000000c2c5d46481c291be111d5e3a0b52114bdf212a01000000
	0000000000000000000000000000000000000000000000000000000001000000
	0000000000000000000000000000000000000000000de0b6b3a7640000000000
	0000000000000000000000000000000000000000000000000000000080000000
	00000000000000000000000000000000000000000000000000000000570c0182
	13dae5f9c236beab905c8305cb159c5fa1aae500d43593c715fdd31c61141abd
	04a99fd6822c8558854ccde39a5684e7a56da27d0000d9e9ac2d780300000000
	0000000000000000000000000000000000000000000000000000000000
"
);

// Message with nonce = 2
const MESSAGE_DATA_1: [u8; 317] = hex!(
	"
	f9013a944130819912a398f4eb84e7f16ed443232ba638b5e1a05e9ae1d7c484
	f74d554a503aa825e823725531d97e784dd9b1aacdb58d1f7076b90100000000
	000000000000000000c2c5d46481c291be111d5e3a0b52114bdf212a01000000
	0000000000000000000000000000000000000000000000000000000002000000
	0000000000000000000000000000000000000000000de0b6b3a7640000000000
	0000000000000000000000000000000000000000000000000000000080000000
	00000000000000000000000000000000000000000000000000000000570c0182
	13dae5f9c236beab905c8305cb159c5fa1aae500d43593c715fdd31c61141abd
	04a99fd6822c8558854ccde39a5684e7a56da27d0000d9e9ac2d780300000000
	0000000000000000000000000000000000000000000000000000000000
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
				data: Default::default(),
			},
		};
		assert_noop!(
			IncentivizedInboundChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidSourceChannel
		);
	});
}

#[test]
fn test_submit() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(IncentivizedInboundChannel::submit(origin.clone(), message_1));
		let nonce: u64 = <Nonce<Test>>::get();
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
		assert_ok!(IncentivizedInboundChannel::submit(origin.clone(), message_2));
		let nonce: u64 = <Nonce<Test>>::get();
		assert_eq!(nonce, 2);
	});
}

#[test]
fn test_submit_with_invalid_nonce() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(IncentivizedInboundChannel::submit(origin.clone(), message.clone()));
		let nonce: u64 = <Nonce<Test>>::get();
		assert_eq!(nonce, 1);

		// Submit the same again
		assert_noop!(
			IncentivizedInboundChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn test_handle_fee() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();

		let _ = Balances::deposit_creating(&SourceAccount::get(), 100000000000); // 10 DOT
		let _ = Balances::deposit_creating(&TreasuryAccount::get(), Balances::minimum_balance());
		let _ = Balances::deposit_creating(&relayer, Balances::minimum_balance());

		let fee = 10000000000; // 1 DOT

		IncentivizedInboundChannel::handle_fee(fee, &relayer);
		assert_eq!(Balances::free_balance(&TreasuryAccount::get()), 2000000001);
		assert_eq!(Balances::free_balance(&relayer), 8000000001);
	});
}

#[test]
fn test_set_reward_fraction_not_authorized() {
	new_tester(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();
		assert_noop!(
			IncentivizedInboundChannel::set_reward_fraction(
				Origin::signed(bob),
				Perbill::from_percent(60)
			),
			DispatchError::BadOrigin
		);
	});
}
