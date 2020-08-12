#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use sp_std::prelude::*;
use sp_core::{H160, U256, RuntimeDebug};
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
};

use codec::{Encode, Decode};

use artemis_core::{AppID, Application, RelayEventEmitter, Message};
use artemis_ethereum::{self as ethereum, SignedMessage};
use artemis_generic_asset as generic_asset;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const APP_ID: &[u8; 32] = &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum RelayEvent<AccountId> {
	Burned(H160, AccountId, U256)
}

pub trait Trait: system::Trait + generic_asset::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	type Bridge: RelayEventEmitter<RelayEvent<Self::AccountId>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Erc20Module {}
}

decl_event!(
	pub enum Event {}
);

decl_error! {
	pub enum Error for Module<T: Trait> {}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn burn(origin, token_id: H160, amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			<generic_asset::Module<T>>::do_burn(token_id, &who, amount)?;
			T::Bridge::emit(APP_ID, RelayEvent::<T::AccountId>::Burned(token_id, who, amount));
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
				<generic_asset::Module<T>>::do_mint(token, &account, amount)?;
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
