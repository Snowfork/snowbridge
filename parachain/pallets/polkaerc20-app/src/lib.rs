#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use sp_std::prelude::*;
use sp_core::{H160, U256};
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
};

use codec::Decode;

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
	trait Store for Module<T: Trait> as Erc20Module {}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		TokenId = H160,
	{
		Transfer(TokenId, AccountId, H160, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		InvalidTokenId
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		// Users should burn their holdings to release funds on the Ethereum side
		// TODO: Calculate weights
		#[weight = 0]
		pub fn burn(origin, token_id: H160, recipient: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// The token_id 0 is reserved for the ETH app
			if token_id == H160::zero() {
				return Err(Error::<T>::InvalidTokenId.into())
			}

			<asset::Module<T>>::do_burn(token_id, &who, amount)?;
			Self::deposit_event(RawEvent::Transfer(token_id, who.clone(), recipient, amount));
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	fn bytes_to_account_id(data: &[u8]) -> Option<T::AccountId> {
		T::AccountId::decode(&mut &data[..]).ok()
	}

	fn handle_event(payload: Payload) -> DispatchResult {
		if payload.token_addr.is_zero() {
			return Err(DispatchError::Other("Invalid token address"))
		}

		let account = Self::bytes_to_account_id(&payload.recipient_addr)
			.ok_or(DispatchError::Other("Invalid recipient account"))?;

		<asset::Module<T>>::do_mint(payload.token_addr, &account, payload.amount)?;

		Ok(())
	}

}

impl<T: Trait> Application for Module<T> {

	fn handle(message: Message) -> DispatchResult {
		let payload = Payload::decode(message.payload)
			.map_err(|_| DispatchError::Other("Failed to decode ethereum log"))?;

		Self::handle_event(payload)
	}
}
