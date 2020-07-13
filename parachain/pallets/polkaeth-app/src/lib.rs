#![cfg_attr(not(feature = "std"), no_std)]
///
/// Skeleton implementation for a PolkaETH token
///
/// Resources:
///   https://blog.polymath.network/substrate-deep-dive-imbalances-8dfa89cc1d1
///

use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult,
	traits::{Currency, ExistenceRequirement, WithdrawReasons, WithdrawReason}};

use frame_system::{self as system, ensure_signed };

use sp_std::prelude::*;

type PolkaETH<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

use common::{AppID, Message, Application};

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
	pub enum Event<T> where
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
		fn transfer(origin, to: T::AccountId, amount: PolkaETH<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			T::Currency::transfer(&who, &to, amount, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(RawEvent::Transfer(who, to, amount));
			Ok(())
		}

		///
		/// The parachain will mint PolkaETH for users who have locked up ETH in a bridge smart contract.
		///
		#[weight = 10_000]
		fn mint(origin, amount: PolkaETH<T>) -> DispatchResult {
			// TODO: Ensure only our broker and verifier modules can call this dispatchable.
			let who = ensure_signed(origin)?;

			let _ = T::Currency::deposit_creating(&who, amount);

			Self::deposit_event(RawEvent::Minted(who, amount));

			Ok(())
		}

		///
		/// To initiate a transfer of PolkaETH to ETH, account holders must burn PolkaETH.
		///
		#[weight = 10_000]
		fn burn(origin, amount: PolkaETH<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut reasons = WithdrawReasons::none();
			reasons.set(WithdrawReason::Transfer);

			let _ = T::Currency::withdraw(&who, amount, reasons, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(RawEvent::Burned(who, amount));

			Ok(())
		}

	}
}

impl<T: Trait> Application for Module<T> {

	/// ETH doesnt have a contract address so, we just make our AppID 32 zero bytes.
	fn is_handler_for(app_id: AppID) -> bool {
		app_id == [0; 32]
	}

	fn handle(app_id: AppID, message: Message) -> DispatchResult {
		// We'll most likely want to call Mint() here.
		Ok(())
	}

}
