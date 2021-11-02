#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult, Dispatchable, Parameter},
	traits::EnsureOrigin,
	weights::{extract_actual_weight, GetDispatchInfo},
};

use cumulus_pallet_xcm::{ensure_sibling_para, Origin as CumulusOrigin};
use cumulus_primitives_core::ParaId;

use sp_core::RuntimeDebug;

use sp_core::H160;
use sp_std::prelude::*;

use codec::{Decode, Encode};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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
		type Origin: From<RawOrigin>
			+ From<<Self as frame_system::Config>::Origin>
			+ Into<Result<CumulusOrigin, <Self as Config>::Origin>>;

		/// The overarching dispatch call type.
		type Call: Parameter
			+ GetDispatchInfo
			+ Dispatchable<
				Origin = <Self as Config>::Origin,
				PostInfo = frame_support::dispatch::PostDispatchInfo,
			>;

		#[pallet::constant]
		type BridgeParaId: Get<ParaId>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MessageDispatched(DispatchResult),
	}

	pub enum Error {
		/// Message has been rejected
		MessageRejected,
		/// We have failed to decode a Call from the message.
		MessageDecodeFailed,
	}

	#[pallet::origin]
	pub type Origin = RawOrigin;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
		})]
		pub fn dispatch(
			origin: OriginFor<T>,
			source: H160,
			call: Box<<T as Config>::Call>,
		) -> DispatchResult {
			let para_id = ensure_sibling_para(<T as Config>::Origin::from(origin))?;
			ensure!(para_id == T::BridgeParaId::get(), DispatchError::BadOrigin.into());

			// Get pre-dispatch weight
			let dispatch_info = call.get_dispatch_info();

			// dispatch call
			let origin = RawOrigin(source).into();
			let result = call.dispatch(origin);

			Self::deposit_event(Event::MessageDispatched(result.map(drop).map_err(|e| e.error)));

			let actual_call_weight = extract_actual_weight(&result, &dispatch_info);

			Ok(())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{dispatch::DispatchError, parameter_types};
	use frame_system::{EventRecord, Phase};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	use crate as dispatch2;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
			Dispatch: dispatch2::{Pallet, Storage, Origin, Event<T>},
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
	impl frame_support::traits::Contains<Call> for CallFilter {
		fn contains(call: &Call) -> bool {
			match call {
				Call::System(frame_system::pallet::Call::<Test>::remark(_)) => true,
				_ => false,
			}
		}
	}

	impl dispatch2::Config for Test {
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

			let message = Call::System(<frame_system::Call<Test>>::remark(vec![])).encode();

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
