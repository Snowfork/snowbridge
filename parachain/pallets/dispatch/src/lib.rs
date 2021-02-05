#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::{Parameter, Dispatchable, DispatchResult},
	traits::EnsureOrigin,
	weights::GetDispatchInfo,
};

use sp_runtime::traits::BadOrigin;
use sp_core::RuntimeDebug;

use frame_system::{self as system};
use sp_core::H160;
use sp_std::prelude::*;

use artemis_core::MessageDispatch;

use codec::{Encode, Decode};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Origin(pub H160);

impl From<H160> for Origin {
	fn from(hash: H160) -> Origin {
		Origin(hash)
	}
}

pub struct EnsureEthereumAccount;

impl<OuterOrigin> EnsureOrigin<OuterOrigin> for EnsureEthereumAccount
where
	OuterOrigin: Into<Result<Origin, OuterOrigin>> + From<Origin>
{
	type Success = H160;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into().and_then(|o| Ok(o.0))
	}
}

pub fn ensure_ethereum_account<OuterOrigin>(o: OuterOrigin) -> Result<H160, BadOrigin>
	where OuterOrigin: Into<Result<Origin, OuterOrigin>> + From<Origin>
{
	match o.into() {
		Ok(Origin(account)) => Ok(account),
		_ => Err(BadOrigin),
	}
}

pub trait Config: system::Config {
	type Origin: From<Origin>;

	type MessageId: Parameter;

	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	type Call: Parameter
		+ GetDispatchInfo
		+ Dispatchable<
			Origin = <Self as Config>::Origin,
			PostInfo = frame_support::dispatch::PostDispatchInfo,
		>;
}

decl_storage! {
	trait Store for Module<T: Config> as Dispatch {}
}

decl_event! {
    /// Events for the Bridge module.
	pub enum Event<T> where <T as Config>::MessageId {
		Delivered(MessageId, DispatchResult),
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
		fn deposit_event() = default;
	}
}

pub type MessageIdOf<T> = <T as Config>::MessageId;

impl<T: Config> MessageDispatch<MessageIdOf<T>> for Module<T> {
	fn dispatch(source: H160, id: MessageIdOf<T>, payload: &[u8]) {
		let call = match <T as Config>::Call::decode(&mut &payload[..]) {
			Ok(call) => call,
			Err(_) => {
				frame_support::debug::trace!(target: "dispatch", "Failed to decode Call from message {:?}", id);
				return;
			}
		};

		let origin = Origin(source).into();
		let result = call.dispatch(origin);

		Self::deposit_event(RawEvent::Delivered(
			id,
			result.map(drop).map_err(|e| e.error),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{
		impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
		dispatch::DispatchError,
		weights::Weight
	};
	use frame_system::{EventRecord, Phase};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	type AccountId = u64;
	type Dispatch = Module<Test>;
	type System = frame_system::Module<Test>;

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;

	mod dispatch {
		pub use crate::Event;
	}

	mod origin {
		pub use crate::Origin;
	}

	impl_outer_event! {
		pub enum TestEvent for Test {
			frame_system<T>,
			dispatch<T>,
		}
	}

	impl_outer_origin! {
		pub enum Origin for Test where system = frame_system {
			origin
		}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {
			frame_system::System,
			dispatch::Dispatch,
		}
	}

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}

	impl frame_system::Config for Test {
		type Origin = Origin;
		type Index = u64;
		type Call = Call;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = TestEvent;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = ();
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = ();
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type SS58Prefix = ();
	}

	impl Config for Test {
		type Origin = Origin;
		type Event = TestEvent;
		type MessageId = u64;
		type Call = Call;
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		sp_io::TestExternalities::new(t)
	}

	#[test]
	fn should_dispatch_bridge_message() {
		new_test_ext().execute_with(|| {
			let id = 37;
			let source = H160::repeat_byte(7);

			let message = Call::System(<frame_system::Call<Test>>::remark(vec![])).encode();

			System::set_block_number(1);
			Dispatch::dispatch(source, id, &message);

			assert_eq!(
				System::events(),
				vec![EventRecord {
					phase: Phase::Initialization,
					event: TestEvent::dispatch(Event::<Test>::Delivered(id, Err(DispatchError::BadOrigin))),
					topics: vec![],
				}],
			);
		})
	}



}

