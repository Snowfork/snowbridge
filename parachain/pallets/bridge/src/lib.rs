#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use artemis_core::{AppID, Message, Broker};

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Broker: Broker<<Self as system::Trait>::AccountId>;
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
		Received(AccountId, AppID, Hash),
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
			T::Broker::submit(who.clone(), app_id, message.clone())?;

			let message_hash = T::Hashing::hash_of(&message);
			Self::deposit_event(RawEvent::Received(who, app_id, message_hash));

			Ok(())
		}
	}
}
