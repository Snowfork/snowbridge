// Mock runtime
use sp_std::marker::PhantomData;

use super::*;

use sp_core::{H160, H256};
use frame_support::{
	parameter_types,
	dispatch::{DispatchResult, DispatchError},
};
use sp_runtime::{
	traits::{
		BlakeTwo256, IdentityLookup, IdentifyAccount, Verify,
	}, testing::Header, MultiSignature,
};

use artemis_core::{ChannelId, OutboundRouter, nft::ERC721TokenData};

use crate as erc721_app;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Event<T>},
		Dispatch: artemis_dispatch::{Module, Call, Storage, Origin, Event<T>},
		NftApp: artemis_nft::{Module, Call, Config<T>, Storage},
		ERC721App: erc721_app::{Module, Call, Config, Storage, Event<T>},
	}
);

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
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


impl artemis_nft::Config for Test {
	type TokenId = u64;
	type TokenData = ERC721TokenData;
}

impl artemis_dispatch::Config for Test {
	type Origin = Origin;
	type Event = Event;
	type MessageId = u64;
	type Call = Call;
	type CallFilter = ();
}

pub struct MockOutboundRouter<AccountId>(PhantomData<AccountId>);

impl<AccountId> OutboundRouter<AccountId> for MockOutboundRouter<AccountId> {
	fn submit(channel: ChannelId, _: &AccountId, _: H160, _: &[u8]) -> DispatchResult {
        if channel == ChannelId::Basic {
            return Err(DispatchError::Other("some error!"));
        }
		Ok(())
	}
}

impl erc721_app::Config for Test {
	type Event = Event;
	type OutboundRouter = MockOutboundRouter<Self::AccountId>;
	type CallOrigin = artemis_dispatch::EnsureEthereumAccount;
	type WeightInfo = ();
	type TokenId = <Test as artemis_nft::Config>::TokenId;
	type Nft = NftApp;
}

pub fn new_tester() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let config = erc721_app::GenesisConfig {
		address: H160::repeat_byte(1),
	};
	GenesisBuild::<Test>::assimilate_storage(&config, &mut storage).unwrap();

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
