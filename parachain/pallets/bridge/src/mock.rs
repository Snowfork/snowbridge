
use super::*;

use crate::Config;
use sp_core::H256;
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types,
	weights::Weight,
	dispatch::DispatchError
};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, IdentifyAccount, Verify}, testing::Header, Perbill, MultiSignature
};
use sp_std::convert::From;
use frame_system as system;

use artemis_core::{Application, SourceChannel, SourceChannelConfig};
use artemis_ethereum::Log;

use hex_literal::hex;

impl_outer_origin! {
	pub enum Origin for Test {}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
		system<T>,
        test_events,
    }
}

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = ();
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
// Mock ETH app
pub struct MockETHApp;

impl Application for MockETHApp {
	fn handle(_: &[u8]) -> DispatchResult {
		Ok(())
	}

	fn address() -> H160 {
		hex!["8f5acf5f15d4c3d654a759b96bb674a236c8c0f3"].into()
	}
}

pub struct MockERC20App;

impl Application for MockERC20App {
	fn handle(_: &[u8]) -> DispatchResult {
		Ok(())
	}

	fn address() -> H160 {
		H160::zero()
	}
}


impl Config for Test {
	type Event = TestEvent;
	type Verifier = MockVerifier;
	type AppETH = MockETHApp;
	type AppERC20 = MockERC20App;
	type MessageCommitment = ();
}

pub type System = system::Module<Test>;
pub type Bridge = Module<Test>;


pub fn new_tester() -> sp_io::TestExternalities {
	new_tester_with_config::<Test>(GenesisConfig {
		source_channels: SourceChannelConfig {
			basic: SourceChannel {
				address: H160::zero(),
			},
			incentivized: SourceChannel {
				address: H160::zero(),
			}
		}
	})
}

pub fn new_tester_with_source_channels(basic: H160, incentivized: H160) -> sp_io::TestExternalities {
	new_tester_with_config::<Test>(GenesisConfig {
		source_channels: SourceChannelConfig {
			basic: SourceChannel {
				address: basic,
			},
			incentivized: SourceChannel {
				address: incentivized,
			}
		}
	})
}

pub fn new_tester_with_config<T: Config>(config: GenesisConfig) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<T>().unwrap();

	config.assimilate_storage(&mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
