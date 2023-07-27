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

use snowbridge_core::{Command, OutboundMessage, OutboundQueue as OutboundQueueTrait, ParaId};
use sp_core::{H160, H256};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use xcm::prelude::*;
use xcm_executor::traits::ConvertLocation;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MessageHasher: Hash<Output = H256>;
		type OutboundQueue: OutboundQueueTrait;
		type OwnParaId: Get<ParaId>;
		type WeightInfo: WeightInfo;
		type MaxUpgradeDataSize: Get<u32>;
		type CreateAgentOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		type AgentHashedDescription: ConvertLocation<H256>;
		type UniversalLocation: Get<InteriorMultiLocation>;
		type RelayLocation: Get<MultiLocation>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Upgrade { impl_address: H160, impl_code_hash: H256, params_hash: Option<H256> },
		CreateAgent { agent_id: H256, agent_location: MultiLocation },
	}

	#[pallet::error]
	pub enum Error<T> {
		UpgradeDataTooLarge,
		SubmissionFailed,
		LocationConversionFailed,
	}

	#[pallet::storage]
	pub type Agents<T: Config> = StorageMap<_, Twox64Concat, H256, (), OptionQuery>;

	#[pallet::storage]
	pub type Channels<T: Config> = StorageMap<_, Twox64Concat, ParaId, (), OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::upgrade(params.clone().map_or(0, |d| d.len() as u32)))]
		pub fn upgrade(
			origin: OriginFor<T>,
			impl_address: H160,
			impl_code_hash: H256,
			params: Option<Vec<u8>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				params.clone().map_or(0, |d| d.len() as u32) < T::MaxUpgradeDataSize::get(),
				Error::<T>::UpgradeDataTooLarge
			);

			let params_hash = params.as_ref().map(|p| T::MessageHasher::hash(p));

			let message = OutboundMessage {
				origin: T::OwnParaId::get(),
				command: Command::Upgrade { impl_address, impl_code_hash, params },
			};

			let ticket =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;

			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;

			Self::deposit_event(Event::<T>::Upgrade { impl_address, impl_code_hash, params_hash });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn create_agent(origin: OriginFor<T>) -> DispatchResult {
			let mut agent_location: MultiLocation = T::CreateAgentOrigin::ensure_origin(origin)?;

			// Normalize all locations relative to the relay unless its the relay itself.
			let relay_location = T::RelayLocation::get();
			if agent_location != relay_location {
				agent_location
					.reanchor(&relay_location, T::UniversalLocation::get())
					.or(Err(Error::<T>::LocationConversionFailed))?;
			}

			let agent_id = T::AgentHashedDescription::convert_location(&agent_location)
				.ok_or(Error::<T>::LocationConversionFailed)?;

			if Agents::<T>::contains_key(agent_id) {
				return Ok(());
			}

			let message = OutboundMessage {
				origin: T::OwnParaId::get(),
				command: Command::CreateAgent { agent_id },
			};

			let ticket =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;

			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;

			Agents::<T>::insert(agent_id, ());
			Self::deposit_event(Event::<T>::CreateAgent { agent_location, agent_id });

			Ok(())
		}
	}
}
