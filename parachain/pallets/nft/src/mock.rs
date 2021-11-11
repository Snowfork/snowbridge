#![cfg(test)]

use frame_support::{construct_runtime, parameter_types, traits::Everything};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};

use super::*;

use crate as nft;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = u128;
pub type BlockNumber = u64;

impl frame_system::Config for Test {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type OnSetCode = ();
	type DbWeight = ();
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl Config for Test {
	type TokenId = u64;
	type TokenData = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		NonFungibleTokenModule: nft::{Pallet, Storage, Config<T>},
	}
);

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const TOKEN_ID: <Test as Config>::TokenId = 0;
pub const TOKEN_ID_NOT_EXIST: <Test as Config>::TokenId = 100;

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
