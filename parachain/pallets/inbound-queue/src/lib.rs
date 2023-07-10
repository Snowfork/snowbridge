// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
#![cfg_attr(not(feature = "std"), no_std)]

mod envelope;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[cfg(test)]
mod test;

use codec::DecodeAll;
use frame_support::{
	storage::bounded_btree_set::BoundedBTreeSet,
	traits::fungible::{Inspect, Mutate},
};
use frame_system::ensure_signed;
use snowbridge_core::ParaId;
use sp_core::H160;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom, vec::Vec};

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Verifier: Verifier;

		type Token: Mutate<Self::AccountId>;

		type Reward: Get<BalanceOf<Self>>;

		type XcmSender: SendXcm;

		type WeightInfo: WeightInfo;

		type AllowListLength: Get<u32>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T> {
		MessageReceived { dest: ParaId, nonce: u64, result: MessageDispatchResult },
		AllowListAdded { address: sp_core::H160 },
		AllowListRemoved { address: sp_core::H160 },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Message came from an invalid outbound channel on the Ethereum side.
		InvalidOutboundQueue,
		/// Message has an invalid envelope.
		InvalidEnvelope,
		/// Message has an unexpected nonce.
		InvalidNonce,
		/// Cannot convert location
		InvalidAccountConversion,
		// Allow list is full.
		AllowListFull,
	}

	#[pallet::storage]
	#[pallet::getter(fn peer)]
	pub type AllowList<T: Config> =
		StorageValue<_, BoundedBTreeSet<H160, T::AllowListLength>, ValueQuery>;

	#[pallet::storage]
	pub type Nonce<T: Config> = StorageMap<_, Twox64Concat, ParaId, u64, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub allowlist: Vec<H160>,
	}

	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { allowlist: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			let allowlist: BoundedBTreeSet<H160, T::AllowListLength> =
				BTreeSet::from_iter(self.allowlist.clone().into_iter())
					.try_into()
					.expect("exceeded bound");
			<AllowList<T>>::put(allowlist);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({100_000_000})]
		pub fn submit(origin: OriginFor<T>, message: Message) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// submit message to verifier for verification
			let log = T::Verifier::verify(&message)?;

			// Decode log into an Envelope
			let envelope = Envelope::try_from(log).map_err(|_| Error::<T>::InvalidEnvelope)?;

			// Verify that the message was submitted to us from a known
			// outbound channel on the ethereum side
			let allowlist = <AllowList<T>>::get();
			if !allowlist.contains(&envelope.outbound_queue_address) {
				return Err(Error::<T>::InvalidOutboundQueue.into())
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
						return Ok(())
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
					return Ok(())
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
		pub fn add_allow_list(origin: OriginFor<T>, address: sp_core::H160) -> DispatchResult {
			ensure_root(origin)?;

			let success = <AllowList<T>>::mutate(|allowlist| allowlist.try_insert(address).is_ok());

			if success {
				Self::deposit_event(Event::AllowListAdded { address });

				Ok(())
			} else {
				Err(Error::<T>::AllowListFull.into())
			}
		}

		#[pallet::call_index(2)]
		#[pallet::weight({100_000_000})]
		pub fn remove_allow_list(origin: OriginFor<T>, address: sp_core::H160) -> DispatchResult {
			ensure_root(origin)?;

			let removed = <AllowList<T>>::mutate(|allowlist| allowlist.remove(&address));

			if removed {
				Self::deposit_event(Event::AllowListRemoved { address });
			}

			Ok(())
		}
	}
}
