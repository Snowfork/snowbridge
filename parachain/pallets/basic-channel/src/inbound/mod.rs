mod envelope;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[cfg(test)]
mod test;

use frame_system::ensure_signed;
use snowbridge_core::{ChannelId, Message, MessageDispatch, MessageId, Verifier};
use sp_core::H160;
use sp_std::convert::TryFrom;

use envelope::Envelope;
pub use weights::WeightInfo;

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
		#[pallet::weight(100_000_000)]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

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

			let message_id = MessageId::new(ChannelId::Basic, envelope.nonce);
			T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

			Ok(())
		}
	}
}
