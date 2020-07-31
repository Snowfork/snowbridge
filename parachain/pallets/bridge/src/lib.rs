#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};

use sp_std::prelude::*;

use sp_runtime::traits::Hash;

use common::{AppID, Message, Broker, Bridge};

pub trait Trait: system::Trait {

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Broker: Broker;
}

decl_storage! {

	trait Store for Module<T: Trait> as BridgeModule {
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Hash = <T as frame_system::Trait>::Hash,
	{
		MessageReceived(AccountId, AppID, Hash),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn send(origin, app_id: AppID, message: Message) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			T::Broker::submit(app_id, message.clone())?;
			
			Self::deposit_event(RawEvent::MessageReceived(who, app_id, T::Hashing::hash(message.as_ref())));
			Ok(())
		}

	}
}

impl<T: Trait> Bridge for Module<T> {

	fn dummy() {
		()
	}
}
