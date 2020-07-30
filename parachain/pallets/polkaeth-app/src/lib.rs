#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for a PolkaETH token
///
use frame_system::{self as system, ensure_signed};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	traits::{Currency, ExistenceRequirement, WithdrawReason, WithdrawReasons},
};
use sp_std::prelude::*;
use common::{AppID, Application, Message};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type PolkaETH<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type Currency: Currency<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PolkaETHModule {

	}
	add_extra_genesis {
		config(dummy): bool;
		build(|_| {});
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		PolkaETH = PolkaETH<T>,
	{
		Minted(AccountId, PolkaETH),
		Burned(AccountId, PolkaETH),
		Transfer(AccountId, AccountId, PolkaETH),
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

		#[weight = 10_000]
		fn transfer(origin, to: T::AccountId, amount: PolkaETH<T>, allow_death: bool) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let er = match allow_death {
				true => ExistenceRequirement::AllowDeath,
				false => ExistenceRequirement::KeepAlive,
			};

			T::Currency::transfer(&who, &to, amount, er)?;

			Self::deposit_event(RawEvent::Transfer(who, to, amount));
			Ok(())
		}

		///
		/// The parachain will mint PolkaETH for users who have locked up ETH in a bridge smart contract.
		///
		#[weight = 10_000]
		fn mint(origin, to: T::AccountId,  amount: PolkaETH<T>) -> DispatchResult {
			// TODO: verify origin
			let who = ensure_signed(origin)?;

			let _ = T::Currency::deposit_creating(&to, amount);

			Self::deposit_event(RawEvent::Minted(to, amount));

			Ok(())
		}

		///
		/// To initiate a transfer of PolkaETH to ETH, account holders must burn PolkaETH.
		///
		#[weight = 10_000]
		fn burn(origin, amount: PolkaETH<T>, allow_death: bool) -> DispatchResult {
			// TODO: verify origin
			let who = ensure_signed(origin)?;

			// TODO: Add our own reason, the existing ones don't seem to match our needs.
			let mut reasons = WithdrawReasons::none();
			reasons.set(WithdrawReason::Transfer);

			let er = match allow_death {
				true => ExistenceRequirement::AllowDeath,
				false => ExistenceRequirement::KeepAlive,
			};

			let _ = T::Currency::withdraw(&who, amount, reasons, er)?;

			Self::deposit_event(RawEvent::Burned(who, amount));

			Ok(())
		}

	}
}

impl<T: Trait> Application for Module<T> {

	fn handle(app_id: AppID, message: Message) -> DispatchResult {
		// We'll most likely want to call Mint() here.
		Ok(())
	}
}
