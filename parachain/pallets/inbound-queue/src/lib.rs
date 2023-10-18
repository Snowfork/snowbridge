// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
#![cfg_attr(not(feature = "std"), no_std)]

mod envelope;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(feature = "runtime-benchmarks")]
use snowbridge_beacon_primitives::CompactExecutionHeader;
#[cfg(feature = "runtime-benchmarks")]
use snowbridge_ethereum::H256;

pub mod weights;

#[cfg(test)]
mod test;

use codec::{Decode, DecodeAll, Encode};
use frame_support::{
	traits::fungible::{Inspect, Mutate},
	PalletError,
};
use frame_system::{ensure_signed, EnsureRoot};
use scale_info::TypeInfo;
use sp_core::H160;
use sp_runtime::traits::AccountIdConversion;
use sp_std::convert::TryFrom;
use xcm::v3::{
	send_xcm, Junction::*, Junctions::*, MultiLocation, SendError as XcmpSendError, SendXcm,
	XcmHash,
};

use envelope::Envelope;
use snowbridge_core::{
	inbound::{Message, Verifier},
	BridgeModule, OperatingMode, OperatingModeError, ParaId,
};
use snowbridge_router_primitives::{
	inbound,
	inbound::{ConvertMessage, ConvertMessageError},
};
pub use weights::WeightInfo;

type BalanceOf<T> =
	<<T as pallet::Config>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

pub use pallet::*;

pub const LOG_TARGET: &str = "snowbridge-inbound-queue";

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{pallet_prelude::*, traits::tokens::Preservation};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<T> {
		fn initialize_storage(block_hash: H256, header: CompactExecutionHeader);
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The verifier for inbound messages from Ethereum
		type Verifier: Verifier;

		/// Message relayers are rewarded with this asset
		type Token: Mutate<Self::AccountId>;

		/// The amount to reward message relayers
		type Reward: Get<BalanceOf<Self>>;

		/// XCM message sender
		type XcmSender: SendXcm;

		type WeightInfo: WeightInfo;

		// Gateway contract address
		#[pallet::constant]
		type GatewayAddress: Get<H160>;

		/// Convert inbound message to XCM
		type MessageConverter: ConvertMessage;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: BenchmarkHelper<Self>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		/// A message was received from Ethereum
		MessageReceived {
			/// The destination parachain
			dest: ParaId,
			/// The message nonce
			nonce: u64,
			/// XCM hash
			xcm_hash: XcmHash,
		},
		/// Set OperatingMode
		OperatingModeChanged { operating_mode: OperatingMode },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidGateway,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
		/// Message has an invalid payload.
		InvalidPayload,
		/// The max nonce for the type has been reached
		MaxNonceReached,
		/// Cannot convert location
		InvalidAccountConversion,
		/// XCMP send failure
		Send(SendError),
		/// Operational mode errors
		OperationalMode(OperatingModeError),
		/// Message conversion error
		ConvertMessage(ConvertMessageError),
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, TypeInfo, PalletError)]
	pub enum SendError {
		NotApplicable,
		NotRoutable,
		Transport,
		DestinationUnsupported,
		ExceedsMaxMessageSize,
		MissingArgument,
		Fees,
	}

	impl<T: Config> From<XcmpSendError> for Error<T> {
		fn from(e: XcmpSendError) -> Self {
			match e {
				XcmpSendError::NotApplicable => Error::<T>::Send(SendError::NotApplicable),
				XcmpSendError::Unroutable => Error::<T>::Send(SendError::NotRoutable),
				XcmpSendError::Transport(_) => Error::<T>::Send(SendError::Transport),
				XcmpSendError::DestinationUnsupported =>
					Error::<T>::Send(SendError::DestinationUnsupported),
				XcmpSendError::ExceedsMaxMessageSize =>
					Error::<T>::Send(SendError::ExceedsMaxMessageSize),
				XcmpSendError::MissingArgument => Error::<T>::Send(SendError::MissingArgument),
				XcmpSendError::Fees => Error::<T>::Send(SendError::Fees),
			}
		}
	}

	/// The current nonce for each parachain
	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

	/// The current operating mode of the pallet.
	///
	/// Depending on the mode either all, or no transactions will be allowed.
	#[pallet::storage]
	pub type PalletOperatingMode<T: Config> = StorageValue<_, OperatingMode, ValueQuery>;

	impl<T: Config> BridgeModule<T> for Pallet<T> {
		type OperatingMode = OperatingMode;
		type OperatingModeStorage = PalletOperatingMode<T>;
		type AllowedHaltOrigin = EnsureRoot<T::AccountId>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit an inbound message originating from the Gateway contract on Ethereum
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::submit())]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			Self::ensure_not_halted().map_err(Error::<T>::OperationalMode)?;
			let who = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted from the known Gateway contract
			ensure!(T::GatewayAddress::get() == envelope.gateway, Error::<T>::InvalidGateway,);

			// Verify message nonce
			<Nonce<T>>::try_mutate(envelope.dest, |nonce| -> DispatchResult {
				if *nonce == u64::MAX {
					return Err(Error::<T>::MaxNonceReached.into())
				}
				if envelope.nonce != nonce.saturating_add(1) {
					Err(Error::<T>::InvalidNonce.into())
				} else {
					*nonce = nonce.saturating_add(1);
					Ok(())
				}
			})?;

			// Reward relayer from the sovereign account of the destination parachain
			// Expected to fail if sovereign account has no funds
			let sovereign_account = envelope.dest.into_account_truncating();
			T::Token::transfer(&sovereign_account, &who, T::Reward::get(), Preservation::Preserve)?;

			// Decode message into XCM
			let xcm = match inbound::VersionedMessage::decode_all(&mut envelope.payload.as_ref()) {
				Ok(message) => T::MessageConverter::convert(message)
					.map_err(|e| Error::<T>::ConvertMessage(e))?,
				Err(_) => return Err(Error::<T>::InvalidPayload.into()),
			};

			// Attempt to send XCM to a dest parachain
			let dest = MultiLocation { parents: 1, interior: X1(Parachain(envelope.dest.into())) };
			let (xcm_hash, _) = send_xcm::<T::XcmSender>(dest, xcm).map_err(Error::<T>::from)?;

			Self::deposit_event(Event::MessageReceived {
				dest: envelope.dest,
				nonce: envelope.nonce,
				xcm_hash,
			});

			Ok(())
		}

		/// Halt or resume all pallet operations.
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::call_index(1)]
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_operating_mode(
			origin: OriginFor<T>,
			operating_mode: OperatingMode,
		) -> DispatchResult {
			<Self as BridgeModule<_>>::set_operating_mode(origin, operating_mode)?;
			Self::deposit_event(Event::OperatingModeChanged { operating_mode });
			Ok(())
		}
	}
}
