#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult, Dispatchable, Parameter},
	traits::{Contains, EnsureOrigin},
	weights::{extract_actual_weight, GetDispatchInfo},
};

use scale_info::TypeInfo;
use sp_core::RuntimeDebug;

use sp_core::H160;
use sp_std::prelude::*;

use xcm::{latest::prelude::*, latest::NetworkId};

use xcm_executor::traits::InvertLocation;

use snowbridge_core::{MessageDispatch, MessageDispatchInfo, MessageDispatchKind};

use codec::{Decode, Encode};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct RawOrigin(pub H160);

impl From<H160> for RawOrigin {
	fn from(hash: H160) -> RawOrigin {
		RawOrigin(hash)
	}
}

pub struct EnsureEthereumAccount;

impl<OuterOrigin> EnsureOrigin<OuterOrigin> for EnsureEthereumAccount
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>> + From<RawOrigin>,
{
	type Success = H160;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into().and_then(|o| Ok(o.0))
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> OuterOrigin {
		OuterOrigin::from(RawOrigin(H160::repeat_byte(2)))
	}
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The overarching origin type.
		type Origin: From<RawOrigin>;

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

		/// The pallet will filter all incoming calls right before they're dispatched. If this
		/// filter rejects the call, special event (`Event::MessageRejected`) is emitted.
		type CallFilter: Contains<<Self as Config>::Call>;

		/// The type used to actually dispatch an XCM to its destination.
		type XcmRouter: SendXcm;

		/// Means of inverting a location.
		type LocationInverter: InvertLocation;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Message has been dispatched with given result.
		MessageDispatched(T::MessageId, MessageDispatchKind, DispatchResult),
		/// Message has been forwarded using xcmp
		MessageForwarded(T::MessageId, u32),
		/// Message forwarding failed
		MessageForwardingFailed(T::MessageId, u32, XcmError),
		/// Message has been rejected
		MessageRejected(T::MessageId),
		/// We have failed to decode a Call from the message.
		MessageDecodeFailed(T::MessageId),
	}

	#[pallet::origin]
	pub type Origin = RawOrigin;

	pub type MessageIdOf<T> = <T as Config>::MessageId;

	impl<T: Config> MessageDispatch<T, MessageIdOf<T>> for Pallet<T> {
		fn dispatch(
			source: H160,
			id: MessageIdOf<T>,
			dispatch_info: MessageDispatchInfo,
			payload: &[u8],
		) -> DispatchResult {
			match dispatch_info.kind {
				MessageDispatchKind::Local => Self::dispatch_locally(source, id, payload),
				MessageDispatchKind::Remote { para_id } => {
					Self::dispatch_remotely(source, id, para_id, dispatch_info.weight, payload)
				}
			}
		}

		fn dispatch_locally(source: H160, id: MessageIdOf<T>, payload: &[u8]) -> Option<Weight> {
			let call = match <T as Config>::Call::decode(&mut &payload[..]) {
				Ok(call) => call,
				Err(_) => {
					Self::deposit_event(Event::MessageDecodeFailed(id));
					return None;
				}
			};

			if !T::CallFilter::contains(&call) {
				Self::deposit_event(Event::MessageRejected(id));
				return None;
			}

			// Get pre-dispatch weight
			let dispatch_info = call.get_dispatch_info();

			// dispatch call
			let origin = RawOrigin(source).into();
			let result = call.dispatch(origin);

			Self::deposit_event(Event::MessageDispatched(
				id,
				result.map(drop).map_err(|e| e.error),
			));

			let actual_call_weight = extract_actual_weight(&result, &dispatch_info);

			Some(actual_call_weight)
		}

		fn dispatch_remotely(
			source: H160,
			id: MessageIdOf<T>,
			para_id: u32,
			weight: u64,
			payload: &[u8],
		) {
			let message = RelayedFrom {
				who: Junctions::X1(AccountKey20 { network: NetworkId::Any, key: source.into() }),
				message: Box::new(Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: weight,
					call: payload.to_vec().into(),
				}),
			};

			let dest = MultiLocation { parents: 1, interior: X1(Parachain(para_id)) };
			match T::XcmRouter::send_xcm(dest, message) {
				Ok(()) => Self::deposit_event(Event::MessageForwarded(id, para_id)),
				Err(error) => {
					Self::deposit_event(Event::MessageForwardingFailed(id, para_id, error))
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{dispatch::DispatchError, parameter_types, traits::Everything};
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
		type BaseCallFilter = Everything;
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type SS58Prefix = ();
		type OnSetCode = ();
	}

	pub struct CallFilter;
	impl frame_support::traits::Contains<Call> for CallFilter {
		fn contains(call: &Call) -> bool {
			match call {
				Call::System(frame_system::pallet::Call::<Test>::remark { remark: _ }) => true,
				_ => false,
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
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		sp_io::TestExternalities::new(t)
	}

	#[test]
	fn test_dispatch_bridge_message() {
		new_test_ext().execute_with(|| {
			let id = 37;
			let source = H160::repeat_byte(7);

			let message = Call::System(frame_system::Call::remark { remark: vec![] }).encode();

			System::set_block_number(1);
			Dispatch::dispatch(source, id, &message);

			assert_eq!(
				System::events(),
				vec![EventRecord {
					phase: Phase::Initialization,
					event: Event::Dispatch(crate::Event::<Test>::MessageDispatched(
						id,
						Err(DispatchError::BadOrigin)
					)),
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

			let message = Call::System(frame_system::Call::set_code { code: vec![] }).encode();

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
