use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	traits::{
		Currency, Get, ExistenceRequirement::KeepAlive,
		WithdrawReasons, Imbalance,
		EnsureOrigin,
	},
	storage::StorageValue,
	log,
	weights::Weight,
};
use frame_system::{self as system, ensure_signed};
use sp_core::{U256, H160};
use sp_std::prelude::*;
use sp_std::convert::TryFrom;
use snowbridge_core::{
	ChannelId, Message, MessageId,
	MessageDispatch, Verifier,
};

use envelope::Envelope;

use sp_runtime::{Perbill, traits::{Zero, Convert}};

mod benchmarking;

#[cfg(test)]
mod test;

mod envelope;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::PositiveImbalance;

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn submit() -> Weight;
	fn set_reward_fraction() -> Weight;
}

impl WeightInfo for () {
	fn submit() -> Weight { 0 }
	fn set_reward_fraction() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<Self, MessageId>;

	type Currency: Currency<Self::AccountId>;

	/// Source of funds to pay relayers
	type SourceAccount: Get<Self::AccountId>;

	/// Treasury Account
	type TreasuryAccount: Get<Self::AccountId>;

	type FeeConverter: Convert<U256, BalanceOf<Self>>;

	/// The origin which may update reward related params
	type UpdateOrigin: EnsureOrigin<Self::Origin>;

	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as IncentivizedInboundModule {
		pub SourceChannel get(fn source_channel) config(): H160;
		pub Nonce: u64;
		pub RewardFraction get(fn reward_fraction) config(): Perbill;

	}
}

decl_event! {
	pub enum Event {
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidSourceChannel,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = T::WeightInfo::submit()]
		pub fn submit(origin, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope: Envelope<T> = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			if envelope.channel != SourceChannel::get() {
				return Err(Error::<T>::InvalidSourceChannel.into())
			}

			// Verify message nonce
			Nonce::try_mutate(|nonce| -> DispatchResult {
				if envelope.nonce != *nonce + 1 {
					Err(Error::<T>::InvalidNonce.into())
				} else {
					*nonce += 1;
					Ok(())
				}
			})?;

			Self::handle_fee(envelope.fee, &relayer);

			let message_id = MessageId::new(ChannelId::Incentivized, envelope.nonce);
			T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

			Ok(())
		}

		#[weight = T::WeightInfo::set_reward_fraction()]
		pub fn set_reward_fraction(origin, fraction: Perbill) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			RewardFraction::set(fraction);
			Ok(())
		}

	}
}

impl<T: Config> Module<T> {
	/*
	* Pay the message submission fee into the relayer and treasury account.
	*
	* - If the fee is zero, do nothing
	* - Otherwise, withdraw the fee amount from the DotApp module account, returning a negative imbalance
	* - Figure out the fraction of the fee amount that should be paid to the relayer
	* - Pay the relayer if their account exists, returning a positive imbalance.
	* - Adjust the negative imbalance by offsetting the amount paid to the relayer
	* - Resolve the negative imbalance by depositing it into the treasury account
	*/
	fn handle_fee(amount: BalanceOf<T>, relayer: &T::AccountId) {
		if amount.is_zero() {
			return;
		}

		let imbalance = match T::Currency::withdraw(&T::SourceAccount::get(), amount, WithdrawReasons::TRANSFER, KeepAlive) {
			Ok(imbalance) => imbalance,
			Err(err) => {
				log::error!("Unable to withdraw from source account: {:?}", err);
				return;
			}
		};

		let reward_fraction: Perbill = RewardFraction::get();
		let reward_amount = reward_fraction.mul_ceil(amount);

		let rewarded = T::Currency::deposit_into_existing(relayer, reward_amount)
			.unwrap_or_else(|_| PositiveImbalanceOf::<T>::zero());

		let adjusted_imbalance = match imbalance.offset(rewarded).same() {
			Ok(imbalance) => imbalance,
			Err(_) => {
				log::error!("Unable to offset imbalance");
				return;
			}
		};

		T::Currency::resolve_creating(&T::TreasuryAccount::get(), adjusted_imbalance);
	}

}
