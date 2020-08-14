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

use codec::{Decode};

use artemis_core::{AppID, Application, Message};
use artemis_ethereum::{self as ethereum, SignedMessage};
use artemis_asset as asset;

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
		Transfer(TokenId, AccountId, U256),
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
		pub fn burn(origin, token_id: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// The token_id 0 is reserved for the PolkaETH app
			if token_id == H160::zero() {
				return Err(Error::<T>::InvalidTokenId.into())
			}

			<asset::Module<T>>::do_burn(token_id, &who, amount)?;
			Self::deposit_event(RawEvent::Transfer(token_id, who.clone(), amount));
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {

	fn bytes_to_account_id(data: &[u8]) -> Option<T::AccountId> {
		T::AccountId::decode(&mut &data[..]).ok()
	}

	fn handle_event(event: ethereum::Event) -> DispatchResult {

		match event {
			ethereum::Event::SendERC20 { recipient, token, amount, ..} => {
				if token.is_zero() {
					return Err(DispatchError::Other("Invalid token address"))
				}
				let account = match Self::bytes_to_account_id(&recipient) {
					Some(account) => account,
					None => {
						return Err(DispatchError::Other("Invalid sender account"))
					}
				};
				<asset::Module<T>>::do_mint(token, &account, amount)?;
				Ok(())
			}
			_ => {
				// Ignore all other ethereum events. In the next milestone the
				// application will only receive messages it is registered to handle
				Ok(())
			}
		}
	}

}

impl<T: Trait> Application for Module<T> {

	fn handle(_app_id: AppID, message: Message) -> DispatchResult {
		let sm = match SignedMessage::decode(&mut message.as_slice()) {
			Ok(sm) => sm,
			Err(_) => return Err(DispatchError::Other("Failed to decode event"))
		};

		let event = match ethereum::Event::decode_from_rlp(sm.data) {
			Ok(event) => event,
			Err(_) => return Err(DispatchError::Other("Failed to decode event"))
		};

		Self::handle_event(event)
	}
}
