#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
};
use frame_system::{self as system};
use sp_core::H160;
use sp_std::prelude::*;
use artemis_core::{
	ChannelId, MessageNonce, MessageCommitment,
};

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Used by outbound channels to persist messages for outbound delivery.
	type MessageCommitment: MessageCommitment;

}

decl_storage! {
	trait Store for Module<T: Config> as BasicOutboundModule {
		pub Nonce: u64;
	}
}

decl_event! {
	pub enum Event {
		MessageAccepted(MessageNonce),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;
	}
}

impl<T: Config> Module<T> {
	pub fn submit(_: &T::AccountId, target: H160, payload: &[u8]) -> DispatchResult {
		Nonce::try_mutate(|nonce| -> DispatchResult {
			*nonce += 1;
			T::MessageCommitment::add(ChannelId::Basic, target, *nonce, payload)?;
			<Module<T>>::deposit_event(Event::MessageAccepted(*nonce));
			Ok(())
		})
	}
}
