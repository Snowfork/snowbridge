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

use sp_std::convert::{TryInto};
use sp_std::if_std;

use artemis_ethereum::{self as ethereum};

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
		// Free balances are represent as a doublemap: (TokenAddr, AccountId) -> Balance
		//
		// The choice of hashers was influenced by pallet-generic-asset, where free balances
		// are also represented using a StorageDouble with twox_64_concat and blake2_128_concat
		// hashers. So I'm assuming its a safe choice.
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
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

	}
}

impl<T: Trait> Module<T> {

	fn bytes_to_account_id(data: &[u8]) -> Option<T::AccountId> {
		T::AccountId::decode(&mut &data[..]).ok()
	}

	fn u128_to_balance(input: u128) -> Option<T::Balance>  {
		input.try_into().ok()
	}

	fn do_mint(token_addr: H160, to: &T::AccountId, amount: T::Balance) -> DispatchResult {
		let original_free_balance = <FreeBalance<T>>::get(&token_addr, to);
		let value = original_free_balance.checked_add(&amount)
			.ok_or(Error::<T>::FreeMintingOverflow)?;
		<FreeBalance<T>>::insert(&token_addr, to, value);
		Self::deposit_event(RawEvent::Minted(token_addr, to.clone(), amount));
		Ok(())
	}

	fn handle_event(event: ethereum::Event) -> DispatchResult {
		match event {
			ethereum::Event::SendERC20 { sender, recipient, token, amount, nonce} => {
				let account = match Self::bytes_to_account_id(&recipient) {
					Some(account) => account,
					None => {
						return Err(DispatchError::Other("Invalid sender account"))
					}
				};
				let balance = match Self::u128_to_balance(amount.as_u128()) {
					Some(balance) => balance,
					None => {
						return Err(DispatchError::Other("Invalid amount"))
					}
				};
				Self::do_mint(token, &account, balance)
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

	fn handle(app_id: AppID, message: Message) -> DispatchResult {
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
