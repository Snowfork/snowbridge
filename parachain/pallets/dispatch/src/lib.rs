#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::{Parameter, Dispatchable, DispatchResult},
	traits::{EnsureOrigin, Filter},
	weights::GetDispatchInfo,
};

use sp_core::RuntimeDebug;

use frame_system::{self as system};
use sp_core::H160;
use sp_std::prelude::*;

use snowbridge_core::MessageDispatch;

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

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> OuterOrigin {
		OuterOrigin::from(Origin(H160::repeat_byte(2)))
	}
}

pub trait Config: system::Config {

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;

	/// The overarching origin type.
	type Origin: From<Origin>;

	/// Id of the message. Whenever message is passed to the dispatch module, it emits
	/// event with this id + dispatch result.
	type MessageId: Parameter;

	/// The overarching dispatch call type.
	type Call: Parameter
		+ GetDispatchInfo
		+ Dispatchable<
			Origin = <Self as Config>::Origin,
			PostInfo = frame_support::dispatch::PostDispatchInfo,
		>;

	/// The pallet will filter all incoming calls right before they're dispatched. If this filter
	/// rejects the call, special event (`Event::MessageRejected`) is emitted.
	type CallFilter: Filter<<Self as Config>::Call>;
}

decl_storage! {
	trait Store for Module<T: Config> as Dispatch {}
}

decl_event! {
    /// Events for the Bridge module.
	pub enum Event<T> where <T as Config>::MessageId {
		/// Message has been dispatched with given result.
		MessageDispatched(MessageId, DispatchResult),
		/// Message has been rejected
		MessageRejected(MessageId),
		/// We have failed to decode a Call from the message.
		MessageDecodeFailed(MessageId),
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {
		fn deposit_event() = default;
	}
}

pub type MessageIdOf<T> = <T as Config>::MessageId;

impl<T: Config> MessageDispatch<T, MessageIdOf<T>> for Module<T> {
	fn dispatch(source: H160, id: MessageIdOf<T>, payload: &[u8]) {
		let call = match <T as Config>::Call::decode(&mut &payload[..]) {
			Ok(call) => call,
			Err(_) => {
				Self::deposit_event(RawEvent::MessageDecodeFailed(id));
				return;
			}
		};

		if !T::CallFilter::filter(&call) {
			Self::deposit_event(RawEvent::MessageRejected(id));
			return;
		}

		let origin = Origin(source).into();
		let result = call.dispatch(origin);

		Self::deposit_event(RawEvent::MessageDispatched(
			id,
			result.map(drop).map_err(|e| e.error),
		));
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_dispatch_event(id: MessageIdOf<T>) -> Option<<T as system::Config>::Event> {
		let event: <T as Config>::Event = RawEvent::MessageDispatched(id, Ok(())).into();
		Some(event.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{
		parameter_types,
		dispatch::DispatchError,
	};
	use frame_system::{EventRecord, Phase};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	use crate as dispatch;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
			Dispatch: dispatch::{Pallet, Storage, Origin, Event<T>},
		}
	);

	type AccountId = u64;

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
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
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = ();
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type SS58Prefix = ();
		type OnSetCode = ();
	}

	pub struct CallFilter;
	impl Filter<Call> for CallFilter {
		fn filter(call: &Call) -> bool {
			match call {
				Call::System(frame_system::pallet::Call::<Test>::remark(_)) => true,
				_ => false
			}
		}
	}

	impl dispatch::Config for Test {
		type Origin = Origin;
		type Event = Event;
		type MessageId = u64;
		type Call = Call;
		type CallFilter = CallFilter;
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		sp_io::TestExternalities::new(t)
	}

	#[test]
	fn test_dispatch_bridge_message() {
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
					event: Event::Dispatch(crate::Event::<Test>::MessageDispatched(id, Err(DispatchError::BadOrigin))),
					topics: vec![],
				}],
			);
		})
	}

	#[test]
	fn test_message_decode_failed() {
		new_test_ext().execute_with(|| {
			let id = 37;
			let source = H160::repeat_byte(7);

			let message: Vec<u8> = vec![1, 2, 3];

			System::set_block_number(1);
			Dispatch::dispatch(source, id, &message);

			assert_eq!(
				System::events(),
				vec![EventRecord {
					phase: Phase::Initialization,
					event: Event::Dispatch(crate::Event::<Test>::MessageDecodeFailed(id)),
					topics: vec![],
				}],
			);
		})
	}

	#[test]
	fn test_message_rejected() {
		new_test_ext().execute_with(|| {
			let id = 37;
			let source = H160::repeat_byte(7);

			let message = Call::System(<frame_system::Call<Test>>::set_code(vec![])).encode();

			System::set_block_number(1);
			Dispatch::dispatch(source, id, &message);

			assert_eq!(
				System::events(),
				vec![EventRecord {
					phase: Phase::Initialization,
					event: Event::Dispatch(crate::Event::<Test>::MessageRejected(id)),
					topics: vec![],
				}],
			);
		})
	}


}
