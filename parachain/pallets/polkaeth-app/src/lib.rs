#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for a PolkaETH token
///
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
};
use sp_std::prelude::*;
use sp_core::{H160, U256};

use artemis_core::{Application, Message};
use artemis_asset as asset;

mod payload;
use payload::Payload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Erc20Module {

	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId
	{
		Transfer(AccountId, H160, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		// Users should burn their holdings to release funds on the Ethereum side
		// TODO: Calculate weights
		#[weight = 0]
		pub fn burn(origin, recipient: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<asset::Module<T>>::do_burn(H160::zero(), &who, amount)?;
			Self::deposit_event(RawEvent::Transfer(who.clone(), recipient, amount));
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	fn handle_event(payload: Payload<T::AccountId>) -> DispatchResult {
		<asset::Module<T>>::do_mint(H160::zero(), &payload.recipient_addr, payload.amount)
	}
}

impl<T: Trait> Application for Module<T> {

	fn handle(message: Message) -> DispatchResult {
		let payload = Payload::decode(message.payload)
			.map_err(|_| DispatchError::Other("Failed to decode ethereum log"))?;

		Self::handle_event(payload)
	}
}
