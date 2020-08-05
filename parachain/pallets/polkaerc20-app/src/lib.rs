#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, Parameter,
	dispatch::{DispatchResult, DispatchError},
};
use frame_support::storage::{StorageDoubleMap};
use sp_runtime::traits::{
	CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Saturating, AtLeast32Bit,
	Zero, Bounded, AtLeast32BitUnsigned,
};
use sp_std::prelude::*;
use common::{AppID, Application, Message, SignedMessage};
use codec::{Decode, EncodeLike};
use sp_core::H160;
use sp_std::{fmt::Debug};

use sp_std::convert::TryInto;
use sp_std::if_std;

use artemis_ethereum::Event as EthEvent;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + Debug + MaybeSerializeDeserialize;
}

decl_storage! {
	trait Store for Module<T: Trait> as PolkaERC20Map {		
		pub FreeBalance: double_map hasher(twox_64_concat) H160, hasher(blake2_128_concat) T::AccountId => T::Balance;		
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		BalanceERC20 = <T as Trait>::Balance,
	{
		Minted(H160, AccountId, BalanceERC20),
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

	}
}

impl<T: Trait> Module<T> {

	/// Make an AccountID from the recipient address encoded
	/// in the ethereum event.
	fn make_account_id(data: &[u8]) -> T::AccountId {
		T::AccountId::decode(&mut &data[..]).unwrap_or_default()
	}

	fn u128_to_balance(input: u128) -> Option<T::Balance>  {
		input.try_into().ok()
	}

	pub fn do_mint(token_addr: H160, to: &T::AccountId, amount: T::Balance) -> DispatchResult {
		<FreeBalance<T>>::insert(&token_addr, to, amount);
		Self::deposit_event(RawEvent::Minted(token_addr, to.clone(), amount));
		Ok(())
	}

}

impl<T: Trait> Application for Module<T> {

	fn handle(app_id: AppID, message: Message) -> DispatchResult {

		// TODO: Rather implement From<DecodeError> for DispatchError
		let sm = match SignedMessage::decode(&mut message.as_slice()) {
			Ok(sm) => sm,
			Err(_) => return Err(DispatchError::Other("Failed to decode message"))
		};
		
		let event = match EthEvent::decode_from_rlp(sm.data) {
			Ok(event) => event,
			Err(_) => return Err(DispatchError::Other("Failed to decode message"))
		};
			
		match event {
			EthEvent::SendERC20 { sender, recipient, token, amount, nonce} => {
				let to = Self::make_account_id(&recipient);
				let amt = Self::u128_to_balance(amount.as_u128());
				if let Some(value) = amt {
					Self::do_mint(token, &to, value)?;
				}
			}
			_ => {
				// Ignore all other ethereum events.
				// In the next development milestone the application
				// will only receive messages it can specifically handle.
			}
		}

		Ok(())
	}
}

