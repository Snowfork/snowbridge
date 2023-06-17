// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

use snowbridge_core::{ContractId, OutboundMessage, OutboundQueue as OutboundQueueTrait, ParaId};
use sp_core::{H160, H256, U256};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;

use ethabi::Token;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MessageHasher: Hash<Output = H256>;
		type OutboundQueue: OutboundQueueTrait;
		type OwnParaId: Get<ParaId>;
		type GovernanceProxyContract: Get<ContractId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UpgradeTaskSubmitted { upgrade_task: H160 },
	}

	#[pallet::error]
	pub enum Error<T> {
		SubmissionFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::upgrade())]
		pub fn upgrade(origin: OriginFor<T>, upgrade_task: H160) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			let message = OutboundMessage {
				id: T::MessageHasher::hash(upgrade_task.as_ref()),
				origin: T::OwnParaId::get(),
				gateway: T::GovernanceProxyContract::get(),
				payload: Self::encode_upgrade_payload(upgrade_task),
			};

			let ticket =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;

			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;

			Self::deposit_event(Event::<T>::UpgradeTaskSubmitted { upgrade_task });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn encode_upgrade_payload(upgrade_task: H160) -> Vec<u8> {
			ethabi::encode(&vec![Token::Tuple(vec![
				Token::Uint(U256::from(0u64)),
				Token::Bytes(ethabi::encode(&vec![Token::Tuple(vec![Token::Address(
					upgrade_task,
				)])])),
			])])
		}
	}
}
