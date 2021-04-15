use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
	weights::Weight,
};
use frame_system::{self as system, ensure_signed};
use sp_core::H160;
use sp_std::prelude::*;
use sp_std::convert::TryFrom;
use artemis_core::{
	ChannelId, Message, MessageId,
	MessageDispatch, Verifier,
};

use envelope::Envelope;

mod benchmarking;

#[cfg(test)]
mod test;

mod envelope;

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn submit() -> Weight;
}

impl WeightInfo for () {
	fn submit() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;

	/// Verifier module for message verification.
	type Verifier: Verifier;

	/// Verifier module for message verification.
	type MessageDispatch: MessageDispatch<Self, MessageId>;

	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as BasicInboundModule {
		pub SourceChannel get(fn source_channel) config(): H160;
		pub Nonce: u64;
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
			ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

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

			let message_id = MessageId::new(ChannelId::Basic, envelope.nonce);
			T::MessageDispatch::dispatch(envelope.source, message_id, &envelope.payload);

			Ok(())
		}
	}
}
