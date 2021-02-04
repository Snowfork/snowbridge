#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_variables)]

use frame_support::{
	decl_event, decl_module, decl_storage,
	dispatch::{Parameter, Dispatchable, DispatchResult},
	traits::EnsureOrigin,
	weights::GetDispatchInfo,
	RuntimeDebug,
};

use frame_system::{self as system};
use sp_core::H160;
use sp_std::prelude::*;

use codec::{Encode, Decode};

pub type Origin = RawOrigin;

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct RawOrigin(H160);

impl From<H160> for RawOrigin {
	fn from(hash: H160) -> RawOrigin {
		RawOrigin(hash)
	}
}

pub struct EnsureEthereumAccount;

impl<OuterOrigin> EnsureOrigin<OuterOrigin> for EnsureEthereumAccount
where
	OuterOrigin: Into<Result<RawOrigin, OuterOrigin>>,
{
	type Success = H160;

	fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
		o.into().and_then(|o| Ok(o.0))
	}
}

pub trait Config: system::Config {
	type Origin: From<RawOrigin>;

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

impl<T: Config> Module<T> {

	pub fn dispatch(source: H160, id: MessageIdOf<T>, payload: &[u8]) {
		let call = match <T as Config>::Call::decode(&mut &payload[..]) {
			Ok(call) => call,
			Err(_) => {
				frame_support::debug::trace!(target: "dispatch", "Failed to decode Call from message {:?}", id);
				return;
			}
		};

		let origin = RawOrigin(source).into();
		let result = call.dispatch(origin);

		Self::deposit_event(RawEvent::Delivered(
			id,
			result.map(drop).map_err(|e| e.error),
		));
	}
}
