// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Governance API for controlling the Ethereum side of the bridge
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

use snowbridge_core::{
	outbound::{Command, Message, OutboundQueue as OutboundQueueTrait, ParaId},
	AgentId,
};
use sp_core::{H160, H256};
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use xcm::prelude::*;
use xcm_executor::traits::ConvertLocation;

pub use pallet::*;

pub const LOG_TARGET: &str = "snowbridge-control";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{log, pallet_prelude::*, traits::EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use snowbridge_core::outbound::OperatingMode;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// General-purpose hasher
		type MessageHasher: Hash<Output = H256>;

		/// Send messages to Ethereum
		type OutboundQueue: OutboundQueueTrait;

		/// The ID of this parachain
		type OwnParaId: Get<ParaId>;

		/// Max size of params passed to initializer of the new implementation contract
		type MaxUpgradeDataSize: Get<u32>;

		/// Origin check for `create_agent`
		type CreateAgentOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Converts MultiLocation to H256 in a way that is stable across multiple versions of XCM
		type AgentHashedDescription: ConvertLocation<H256>;

		/// The universal location
		type UniversalLocation: Get<InteriorMultiLocation>;

		/// Location of the relay chain
		type RelayLocation: Get<MultiLocation>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An Upgrade message was sent to the Gateway
		Upgrade { impl_address: H160, impl_code_hash: H256, params_hash: Option<H256> },
		/// An CreateAgent message was sent to the Gateway
		CreateAgent { location: Box<MultiLocation>, agent_id: AgentId },
		/// An CreateChannel message was sent to the Gateway
		CreateChannel { location: Box<MultiLocation>, para_id: ParaId, agent_id: AgentId },
		/// An UpdateChannel message was sent to the Gateway
		UpdateChannel {
			location: Box<MultiLocation>,
			para_id: ParaId,
			agent_id: AgentId,
			mode: OperatingMode,
			fee: u128,
			reward: u128,
		},
		/// An CreateChannel message was sent to the Gateway
		SetOperationMode { mode: OperatingMode },
	}

	#[pallet::error]
	pub enum Error<T> {
		UpgradeDataTooLarge,
		SubmissionFailed,
		LocationReanchorFailed,
		LocationToParaIdConversionFailed,
		LocationToAgentIdConversionFailed,
		AgentAlreadyCreated,
		AgentNotExist,
	}

	#[pallet::storage]
	pub type Agents<T: Config> = StorageMap<_, Twox64Concat, AgentId, (), OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sends a message to the Gateway contract to upgrade itself.
		///
		/// - `origin`: Must be `Root`.
		/// - `impl_address`: The address of the new implementation contract.
		/// - `impl_code_hash`: The codehash of `impl_address`.
		/// - `params`: An optional list of ABI-encoded parameters for the implementation contract's
		///   `initialize(bytes) function. If `None`, the initialization function is not called.
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

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::Upgrade { impl_address, impl_code_hash, params },
			};
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::Upgrade { impl_address, impl_code_hash, params_hash });
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new Agent representing `origin`
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn create_agent(origin: OriginFor<T>) -> DispatchResult {
			let origin_location: MultiLocation = T::CreateAgentOrigin::ensure_origin(origin)?;

			let (agent_id, _, location) = Self::convert_location(origin_location)?;

			log::debug!(
				target: LOG_TARGET,
				"💫 Create Agent request with agent_id {:?}, origin_location at {:?}, location at {:?}",
				agent_id,
				origin_location,
				location
			);

			// Record the agent id or fail if it has already been created
			ensure!(!Agents::<T>::contains_key(agent_id), Error::<T>::AgentAlreadyCreated);

			Agents::<T>::insert(agent_id, ());

			let message =
				Message { origin: T::OwnParaId::get(), command: Command::CreateAgent { agent_id } };
			Self::submit_outbound(message.clone())?;

			log::debug!(
				target: LOG_TARGET,
				"💫 Create Agent request processed with outbound message {:?}",
				message
			);

			Self::deposit_event(Event::<T>::CreateAgent { location: Box::new(location), agent_id });
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new channel for `origin`
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn create_channel(origin: OriginFor<T>) -> DispatchResult {
			let location: MultiLocation = T::CreateAgentOrigin::ensure_origin(origin)?;

			let (agent_id, some_para_id, location) = Self::convert_location(location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);

			let para_id = some_para_id.ok_or(Error::<T>::LocationToParaIdConversionFailed)?;

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::CreateChannel { agent_id, para_id },
			};
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::CreateChannel {
				location: Box::new(location),
				para_id,
				agent_id,
			});

			Ok(())
		}

		/// Sends a message to the Gateway contract to update channel
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn update_channel(
			origin: OriginFor<T>,
			mode: OperatingMode,
			fee: u128,
			reward: u128,
		) -> DispatchResult {
			let location: MultiLocation = T::CreateAgentOrigin::ensure_origin(origin)?;

			let (agent_id, some_para_id, location) = Self::convert_location(location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);

			let para_id = some_para_id.ok_or(Error::<T>::LocationToParaIdConversionFailed)?;

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::UpdateChannel { para_id, mode, fee, reward },
			};
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::UpdateChannel {
				location: Box::new(location),
				para_id,
				agent_id,
				mode,
				fee,
				reward,
			});

			Ok(())
		}

		/// Sends a message to the Gateway contract to set OperationMode
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn set_operating_mode(origin: OriginFor<T>, mode: OperatingMode) -> DispatchResult {
			ensure_root(origin)?;

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::SetOperatingMode { mode },
			};
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::SetOperationMode { mode });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn submit_outbound(message: Message) -> DispatchResult {
			let ticket =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;
			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;
			Ok(())
		}

		pub fn convert_location(
			mut location: MultiLocation,
		) -> Result<(H256, Option<ParaId>, MultiLocation), DispatchError> {
			// Normalize all locations relative to the relay chain.
			let relay_location = T::RelayLocation::get();
			location
				.reanchor(&relay_location, T::UniversalLocation::get())
				.map_err(|_| Error::<T>::LocationReanchorFailed)?;

			let para_id = match location.interior.first() {
				Some(Parachain(index)) => Some((*index).into()),
				_ => None,
			};

			// Hash the location to produce an agent id
			let agent_id = T::AgentHashedDescription::convert_location(&location)
				.ok_or(Error::<T>::LocationToAgentIdConversionFailed)?;

			Ok((agent_id, para_id, location))
		}
	}
}
