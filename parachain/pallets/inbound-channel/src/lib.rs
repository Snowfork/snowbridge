mod envelope;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[cfg(test)]
mod test;

use frame_system::ensure_signed;
use snowbridge_core::{Message, Verifier};
use sp_core::H160;
use sp_std::convert::TryFrom;
use xcm_executor::traits::Convert;

use envelope::Envelope;
pub use weights::WeightInfo;

use xcm::latest::prelude::*;

use frame_support::traits::fungible::{Inspect, Transfer};

type BalanceOf<T> =
        <<T as Config>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Verifier module for message verification.
		type Verifier: Verifier;

		type Token: Transfer<Self::AccountId>;

		type LocationToAccountId: Convert<MultiLocation, Self::AccountId>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;

		type Reward: Get<BalanceOf<Self>>;
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
		/// Cannot convert location
		InvalidAccountConversion
	}

	/// Source channel on the ethereum side
	#[pallet::storage]
	#[pallet::getter(fn source_channel)]
	pub type SourceChannel<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, MultiLocation, u64, ValueQuery>;

	#[pallet::storage]
	pub type LatestVerifiedBlockNumber<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub source_channel: H160,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { source_channel: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<SourceChannel<T>>::put(self.source_channel);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(100_000_000)]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// submit message to verifier for verification
			let (log, block_number) = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			if envelope.channel != <SourceChannel<T>>::get() {
				return Err(Error::<T>::InvalidSourceChannel.into())
			}

			// Verify message nonce
			<Nonce<T>>::try_mutate(envelope.dest, |nonce| -> DispatchResult {
				if envelope.nonce != *nonce + 1 {
					Err(Error::<T>::InvalidNonce.into())
				} else {
					*nonce += 1;
					Ok(())
				}
			})?;

			// Reward relayer from the sovereign account of the destination parachain
			let dest_account = T::LocationToAccountId::convert(envelope.dest).map_err(|_| Error::<T>::InvalidAccountConversion)?;
			T::Token::transfer(&dest_account, &who, T::Reward::get(), true)?;

			<LatestVerifiedBlockNumber<T>>::set(block_number);

			Ok(())
		}
	}
}
