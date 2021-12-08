#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod envelope;
pub mod weights;

#[cfg(test)]
mod test;

use frame_support::{
	log,
	traits::{
		Currency, EnsureOrigin, ExistenceRequirement::KeepAlive, Get, Imbalance, WithdrawReasons,
	},
};
use frame_system::ensure_signed;
use snowbridge_core::{ChannelId, Message, MessageDispatch, MessageId, Verifier};
use sp_core::{H160, U256};
use sp_std::convert::TryFrom;

use envelope::Envelope;
pub use weights::WeightInfo;

use sp_runtime::{
	traits::{Convert, Zero},
	Perbill,
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Verifier module for message verification.
		type Verifier: Verifier;

		/// Verifier module for message verification.
		type MessageDispatch: MessageDispatch<Self, MessageId>;

		type Currency: Currency<Self::AccountId>;

		/// Source of funds to pay relayers
		#[pallet::constant]
		type SourceAccount: Get<Self::AccountId>;

		/// Treasury Account
		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;

		type FeeConverter: Convert<U256, Option<BalanceOf<Self>>>;

		/// The origin which may update reward related params
		type UpdateOrigin: EnsureOrigin<Self::Origin>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	pub enum Event<T> {}

	#[pallet::error]
	pub enum Error<T> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidSourceChannel,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
	}

	/// Source channel on the ethereum side
	#[pallet::storage]
	#[pallet::getter(fn source_channel)]
	pub type SourceChannel<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Fraction of reward going to relayer
	#[pallet::storage]
	#[pallet::getter(fn reward_fraction)]
	pub type RewardFraction<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub source_channel: H160,
		pub reward_fraction: Perbill,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { source_channel: Default::default(), reward_fraction: Perbill::one() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<SourceChannel<T>>::put(self.source_channel);
			<RewardFraction<T>>::put(self.reward_fraction);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100_000_000)]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			let relayer = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope: Envelope<T> =
				Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			if envelope.channel != <SourceChannel<T>>::get() {
				return Err(Error::<T>::InvalidSourceChannel.into())
			}

			// Verify message nonce
			<Nonce<T>>::try_mutate(|nonce| -> DispatchResult {
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

		#[pallet::weight(T::WeightInfo::set_reward_fraction())]
		pub fn set_reward_fraction(origin: OriginFor<T>, fraction: Perbill) -> DispatchResult {
			T::UpdateOrigin::ensure_origin(origin)?;
			<RewardFraction<T>>::set(fraction);
			Ok(())
		}
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::PositiveImbalance;

	impl<T: Config> Pallet<T> {
		/*
		 * Pay the message submission fee into the relayer and treasury account.
		 *
		 * - If the fee is zero, do nothing
		 * - Otherwise, withdraw the fee amount from the DotApp module account, returning a
		 *   negative imbalance
		 * - Figure out the fraction of the fee amount that should be paid to the relayer
		 * - Pay the relayer if their account exists, returning a positive imbalance.
		 * - Adjust the negative imbalance by offsetting the amount paid to the relayer
		 * - Resolve the negative imbalance by depositing it into the treasury account
		 */
		pub(super) fn handle_fee(amount: BalanceOf<T>, relayer: &T::AccountId) {
			if amount.is_zero() {
				return
			}

			let imbalance = match T::Currency::withdraw(
				&T::SourceAccount::get(),
				amount,
				WithdrawReasons::TRANSFER,
				KeepAlive,
			) {
				Ok(imbalance) => imbalance,
				Err(err) => {
					log::error!("Unable to withdraw from source account: {:?}", err);
					return
				},
			};

			let reward_fraction: Perbill = <RewardFraction<T>>::get();
			let reward_amount = reward_fraction.mul_ceil(amount);

			let rewarded = T::Currency::deposit_into_existing(relayer, reward_amount)
				.unwrap_or_else(|_| PositiveImbalanceOf::<T>::zero());

			let adjusted_imbalance = match imbalance.offset(rewarded).same() {
				Ok(imbalance) => imbalance,
				Err(_) => {
					log::error!("Unable to offset imbalance");
					return
				},
			};

			T::Currency::resolve_creating(&T::TreasuryAccount::get(), adjusted_imbalance);
		}
	}
}
