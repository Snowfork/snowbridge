use super::*;

use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	parameter_types,
	traits::{
		tokens::fungible::{Inspect, ItemOf, Mutate},
		Everything, GenesisBuild,
	},
	PalletId,
};
use sp_core::{H160, H256};
use sp_keyring::AccountKeyring as Keyring;
use sp_runtime::{
	testing::Header,
	traits::{
		AccountIdConversion, BlakeTwo256, IdentifyAccount, IdentityLookup, Keccak256, Verify,
	},
	MultiSignature,
};
use sp_std::convert::From;

use crate::outbound as incentivized_outbound_channel;

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
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		IncentivizedOutboundChannel: incentivized_outbound_channel::{Pallet, Call, Config<T>, Storage, Event<T>},
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u64 = 1;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

impl pallet_assets::Config for Test {
	type Event = Event;
	type Balance = u128;
	type AssetId = u128;
	type Currency = Balances;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type WeightInfo = ();
	type Extra = ();
}

parameter_types! {
	pub const MaxMessagePayloadSize: u64 = 128;
	pub const MaxMessagesPerCommit: u64 = 5;
}

parameter_types! {
	pub const EtherAssetId: u128 = 0;
	pub const EtherAppPalletId: PalletId = PalletId(*b"etherapp");
}

pub type Ether = ItemOf<Assets, EtherAssetId, AccountId>;

impl incentivized_outbound_channel::Config for Test {
	const INDEXING_PREFIX: &'static [u8] = b"commitment";
	type Event = Event;
	type Hashing = Keccak256;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerCommit = MaxMessagesPerCommit;
	type FeeCurrency = Ether;
	type SetFeeOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type WeightInfo = ();
}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	let config: incentivized_outbound_channel::GenesisConfig<Test> =
		incentivized_outbound_channel::GenesisConfig { interval: 1u64, fee: 100 };
	config.assimilate_storage(&mut storage).unwrap();

	let assets_config: pallet_assets::GenesisConfig<Test> = pallet_assets::GenesisConfig {
		assets: vec![(0, EtherAppPalletId::get().into_account(), true, 1)],
		metadata: vec![],
		accounts: vec![],
	};
	GenesisBuild::<Test>::assimilate_storage(&assets_config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();

	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn test_submit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		// Deposit enough money to cover fees
		Ether::mint_into(&who, 300).unwrap();

		assert_ok!(IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]));
		assert_eq!(<Nonce<Test>>::get(), 1);

		assert_ok!(IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]));
		assert_eq!(<Nonce<Test>>::get(), 2);
	});
}

#[test]
fn test_submit_fees_burned() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		// Deposit enough money to cover fees
		Ether::mint_into(&who, 300).unwrap();

		assert_ok!(IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]));

		assert_eq!(Ether::balance(&who), 200);
	})
}

#[test]
fn test_submit_not_enough_funds() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		Ether::mint_into(&who, 50).unwrap();

		assert_noop!(
			IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]),
			Error::<Test>::NoFunds
		);
	})
}

#[test]
fn test_submit_exceeds_queue_limit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		// Deposit enough money to cover fees
		Ether::mint_into(&who, 1000).unwrap();

		let max_messages = MaxMessagesPerCommit::get();
		(0..max_messages).for_each(|_| {
			IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]).unwrap()
		});

		assert_noop!(
			IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]),
			Error::<Test>::QueueSizeLimitReached,
		);
	})
}

#[test]
fn test_set_fee_not_authorized() {
	new_tester().execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();
		assert_noop!(
			IncentivizedOutboundChannel::set_fee(Origin::signed(bob), 1000),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_submit_exceeds_payload_limit() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		let max_payload_bytes = MaxMessagePayloadSize::get();
		let payload: Vec<u8> = (0..).take(max_payload_bytes as usize + 1).collect();

		assert_noop!(
			IncentivizedOutboundChannel::submit(&who, target, payload.as_slice()),
			Error::<Test>::PayloadTooLarge,
		);
	})
}

#[test]
fn test_submit_fails_on_nonce_overflow() {
	new_tester().execute_with(|| {
		let target = H160::zero();
		let who: AccountId = Keyring::Bob.into();

		<Nonce<Test>>::set(u64::MAX);
		assert_noop!(
			IncentivizedOutboundChannel::submit(&who, target, &vec![0, 1, 2]),
			Error::<Test>::Overflow,
		);
	});
}
