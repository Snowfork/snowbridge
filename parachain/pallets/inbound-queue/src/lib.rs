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

use codec::DecodeAll;
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::ensure_signed;
use snowbridge_core::ParaId;
use sp_core::H160;
use sp_runtime::traits::AccountIdConversion;
use sp_std::convert::TryFrom;

use envelope::Envelope;
use snowbridge_core::{Message, Verifier};
use snowbridge_router_primitives::inbound;

use xcm::v3::{send_xcm, Junction::*, Junctions::*, MultiLocation, SendError};

pub use weights::WeightInfo;

use frame_support::{CloneNoBound, EqNoBound, PartialEqNoBound};

use codec::{Decode, Encode};

use scale_info::TypeInfo;

type BalanceOf<T> =
	<<T as Config>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(CloneNoBound, EqNoBound, PartialEqNoBound, Encode, Decode, Debug, TypeInfo)]
pub enum MessageDispatchResult {
	InvalidPayload,
	Dispatched,
	NotDispatched(SendError),
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::{pallet_prelude::*, traits::tokens::Preservation};
	use frame_system::pallet_prelude::*;
	use xcm::v3::SendXcm;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<T> {
		fn initialize_storage(block_hash: H256, header: CompactExecutionHeader);
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Verifier: Verifier;

		type Token: Mutate<Self::AccountId>;

		type Reward: Get<BalanceOf<Self>>;

		type XcmSender: SendXcm;

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: BenchmarkHelper<Self>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		MessageReceived { dest: ParaId, nonce: u64, result: MessageDispatchResult },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidGateway,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
		/// Cannot convert location
		InvalidAccountConversion,
	}

	#[pallet::storage]
	#[pallet::getter(fn gateway)]
	pub type Gateway<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub gateway: H160,
		#[serde(skip)]
		pub _config: sp_std::marker::PhantomData<T>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { gateway: Default::default(), _config: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Gateway::<T>::put(self.gateway);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::submit())]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted from the known Gateway contract
			ensure!(Gateway::<T>::get() == envelope.gateway, Error::<T>::InvalidGateway,);

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
			// Expected to fail if sovereign account has no funds
			let sovereign_account = envelope.dest.into_account_truncating();
			T::Token::transfer(&sovereign_account, &who, T::Reward::get(), Preservation::Preserve)?;

			// From this point, any errors are masked, i.e the extrinsic will
			// succeed even if the message was not successfully decoded or dispatched.

			// Attempt to decode message
			let decoded_message =
				match inbound::VersionedMessage::decode_all(&mut envelope.payload.as_ref()) {
					Ok(inbound::VersionedMessage::V1(decoded_message)) => decoded_message,
					Err(_) => {
						Self::deposit_event(Event::MessageReceived {
							dest: envelope.dest,
							nonce: envelope.nonce,
							result: MessageDispatchResult::InvalidPayload,
						});
						return Ok(());
					},
				};

			// Attempt to convert to XCM
			let sibling_para =
				MultiLocation { parents: 1, interior: X1(Parachain(envelope.dest.into())) };
			let xcm = match decoded_message.try_into() {
				Ok(xcm) => xcm,
				Err(_) => {
					Self::deposit_event(Event::MessageReceived {
						dest: envelope.dest,
						nonce: envelope.nonce,
						result: MessageDispatchResult::InvalidPayload,
					});
					return Ok(());
				},
			};

			// Attempt to send XCM to a sibling parachain
			match send_xcm::<T::XcmSender>(sibling_para, xcm) {
				Ok(_) => Self::deposit_event(Event::MessageReceived {
					dest: envelope.dest,
					nonce: envelope.nonce,
					result: MessageDispatchResult::Dispatched,
				}),
				Err(err) => Self::deposit_event(Event::MessageReceived {
					dest: envelope.dest,
					nonce: envelope.nonce,
					result: MessageDispatchResult::NotDispatched(err),
				}),
			}

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({100_000_000})]
		pub fn set_gateway(origin: OriginFor<T>, gateway: H160) -> DispatchResult {
			ensure_root(origin)?;
			Gateway::<T>::put(gateway);
			Ok(())
		}
	}
}
