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

use frame_support::traits::fungible::{Inspect, Mutate};
use snowbridge_core::{
	outbound::{
		Command, Message, OperatingMode, OutboundQueue as OutboundQueueTrait, ParaId,
	},
	AgentId,
};
use sp_runtime::{RuntimeDebug, traits::Hash};
use sp_std::prelude::*;
use xcm::prelude::*;
use xcm_executor::traits::ConvertLocation;

pub use pallet::*;
use sp_core::{H160, H256};

pub const LOG_TARGET: &str = "snowbridge-control";

pub type BalanceOf<T> =
	<<T as pallet::Config>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Copy, Clone, PartialEq, RuntimeDebug)]
pub struct OriginInfo {
	/// The location of this origin
	pub location: MultiLocation,
	/// The parachain hosting this origin
	pub para_id: ParaId,
	/// The deterministic ID of the agent for this origin
	pub agent_id: H256,
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		log,
		pallet_prelude::*,
		traits::{tokens::Preservation, EnsureOrigin},
	};
	use frame_system::pallet_prelude::*;

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
		#[pallet::constant]
		type MaxUpgradeDataSize: Get<u32>;

		/// Implementation that ensures origin is an XCM location for agent operations
		type AgentOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Implementation that ensures origin is an XCM location for channel operations
		type ChannelOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Converts MultiLocation to H256 in a way that is stable across multiple versions of XCM
		type AgentIdOf: ConvertLocation<H256>;

		/// The universal location
		type UniversalLocation: Get<InteriorMultiLocation>;

		/// Location of the relay chain
		type RelayLocation: Get<MultiLocation>;

		/// Token reserved for control operations
		type Token: Mutate<Self::AccountId>;

		/// TreasuryAccount to collect fees
		type TreasuryAccount: Get<Self::AccountId>;

		/// Converts MultiLocation to a sovereign account
		type SovereignAccountOf: ConvertLocation<Self::AccountId>;

		type CreateAgentDeposit: Get<BalanceOf<T>>;
		type CreateChannelDeposit: Get<BalanceOf<T>>;

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
		CreateChannel { para_id: ParaId, agent_id: AgentId },
		/// An UpdateChannel message was sent to the Gateway
		UpdateChannel {
			para_id: ParaId,
			agent_id: AgentId,
			mode: OperatingMode,
			fee: u128,
			reward: u128,
		},
		/// An SetOperatingMode message was sent to the Gateway
		SetOperatingMode { mode: OperatingMode },
		/// An TransferNativeFromAgent message was sent to the Gateway
		TransferNativeFromAgent { agent_id: AgentId, recipient: H160, amount: u128 },
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
		ChannelAlreadyCreated,
		ChannelNotExist,
		LocationToSovereignAccountConversionFailed,
		EstimateFeeFailed,
		ChargeFeeFailed,
	}

	#[pallet::storage]
	pub type Agents<T: Config> = StorageMap<_, Twox64Concat, AgentId, (), OptionQuery>;

	#[pallet::storage]
	pub type Channels<T: Config> = StorageMap<_, Twox64Concat, ParaId, (), OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sends a message to the Gateway contract to upgrade itself.
		///
		/// - `origin`: Must be `Root`.
		/// - `impl_address`: The address of the new implementation contract.
		/// - `impl_code_hash`: The codehash of `impl_address`.
		/// - `params`: An optional list of ABI-encoded parameters for the implementation contract's
		///   `initialize(bytes) function. If `None`, the initialization function is not called.
		/// - `maximum_required_gas`: Maximum amount of gas required by the Gateway.upgrade() handler
		///   to execute this upgrade. This also includes the gas consumed by the `initialize(bytes)` handler of the new
		///   implementation contract.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::upgrade(params.clone().map_or(0, |d| d.len() as u32)))]
		pub fn upgrade(
			origin: OriginFor<T>,
			impl_address: H160,
			impl_code_hash: H256,
			params: Option<Vec<u8>>,
			maximum_required_gas: u64,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				params.clone().map_or(0, |d| d.len() as u32) < T::MaxUpgradeDataSize::get(),
				Error::<T>::UpgradeDataTooLarge
			);

			let params_hash = params.as_ref().map(|p| T::MessageHasher::hash(p));

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::Upgrade { impl_address, impl_code_hash, params, maximum_required_gas },
			};
			Self::submit_outbound(message, MultiLocation::parent())?;

			Self::deposit_event(Event::<T>::Upgrade { impl_address, impl_code_hash, params_hash });
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new Agent representing `origin`
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn create_agent(origin: OriginFor<T>) -> DispatchResult {
			let origin_location: MultiLocation = T::AgentOrigin::ensure_origin(origin)?;

			Self::charge_deposit(&origin_location, T::CreateAgentDeposit::get());

			let OriginInfo { agent_id, .. } =
				Self::process_origin_location(origin_location)?;

			log::debug!(
				target: LOG_TARGET,
				"💫 Create Agent request with agent_id {:?}, origin_location at {:?}, location at {:?}",
				agent_id,
				origin_location,
				location.clone()
			);

			// Record the agent id or fail if it has already been created
			ensure!(!Agents::<T>::contains_key(agent_id), Error::<T>::AgentAlreadyCreated);

			Agents::<T>::insert(agent_id, ());

			let message =
				Message { origin: T::OwnParaId::get(), command: Command::CreateAgent { agent_id } };
			Self::submit_outbound(message.clone(), location)?;

			log::debug!(
				target: LOG_TARGET,
				"💫 Create Agent request processed with outbound message {:?}",
				message
			);

			Self::deposit_event(Event::<T>::CreateAgent { location: Box::new(location), agent_id });
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new Channel representing `origin`
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_channel())]
		pub fn create_channel(origin: OriginFor<T>) -> DispatchResult {
			let location: MultiLocation = T::ChannelOrigin::ensure_origin(origin)?;

			Self::charge_deposit(origin_location, T::CreateChannelDeposit::get());

			let OriginInfo { para_id, agent_id, .. } =
				Self::process_origin_location(location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);
			ensure!(!Channels::<T>::contains_key(para_id), Error::<T>::ChannelAlreadyCreated);

			Channels::<T>::insert(para_id, ());

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::CreateChannel { agent_id, para_id },
			};
			Self::submit_outbound(message, location)?;
			Self::deposit_event(Event::<T>::CreateChannel { para_id, agent_id });

			Ok(())
		}

		/// Sends a message to the Gateway contract to update channel
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_channel())]
		pub fn update_channel(
			origin: OriginFor<T>,
			mode: OperatingMode,
			fee: u128,
			reward: u128,
		) -> DispatchResult {
			let location: MultiLocation = T::ChannelOrigin::ensure_origin(origin)?;

			let OriginInfo { agent_id, para_id, location } =
				Self::process_origin_location(location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);
			ensure!(Channels::<T>::contains_key(para_id), Error::<T>::ChannelNotExist);

			let message = Message {
				origin: para_id,
				command: Command::UpdateChannel { para_id, mode, fee, reward },
			};
			Self::submit_outbound(message, location)?;
			Self::deposit_event(Event::<T>::UpdateChannel { para_id, agent_id, mode, fee, reward });

			Ok(())
		}

		/// Sends a message to the Gateway contract to set OperationMode
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::set_operating_mode())]
		pub fn set_operating_mode(origin: OriginFor<T>, mode: OperatingMode) -> DispatchResult {
			ensure_root(origin)?;

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::SetOperatingMode { mode },
			};
			Self::submit_outbound(message, MultiLocation::parent())?;

			Self::deposit_event(Event::<T>::SetOperatingMode { mode });

			Ok(())
		}

		/// Sends a message to the Gateway contract to transfer asset from agent
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::transfer_native_from_agent())]
		pub fn transfer_native_from_agent(
			origin: OriginFor<T>,
			recipient: H160,
			amount: u128,
		) -> DispatchResult {
			let location: MultiLocation = T::AgentOrigin::ensure_origin(origin)?;

			let OriginInfo { agent_id, para_id, location } =
				Self::process_origin_location(location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);

			let message = Message {
				origin: para_id,
				command: Command::TransferNativeFromAgent { agent_id, recipient, amount },
			};
			Self::submit_outbound(message, location)?;

			Self::deposit_event(Event::<T>::TransferNativeFromAgent {
				agent_id,
				recipient,
				amount,
			});

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn submit_outbound(message: Message, origin_location: MultiLocation) -> DispatchResult {
			let ticket =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;
			Self::charge_fees(&message, &origin_location)?;
			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;
			Ok(())
		}

		pub fn process_origin_location(
			mut location: MultiLocation,
		) -> Result<OriginInfo, DispatchError> {
			// Normalize all locations relative to the relay chain.
			let relay_location = T::RelayLocation::get();
			location
				.reanchor(&relay_location, T::UniversalLocation::get())
				.map_err(|_| Error::<T>::LocationReanchorFailed)?;

			let para_id = match location.interior.first() {
				Some(Parachain(index)) => Some((*index).into()),
				_ => None,
			}
			.ok_or(Error::<T>::LocationToParaIdConversionFailed)?;

			// Hash the location to produce an agent id
			let agent_id = T::AgentIdOf::convert_location(&location)
				.ok_or(Error::<T>::LocationToAgentIdConversionFailed)?;

			Ok(OriginInfo { location, para_id, agent_id, })
		}

		pub fn charge_deposit(origin_location: &MultiLocation, amount: BalanceOf<T>) -> DispatchResult {
			if amount == 0 {
				return Ok(());
			}

			let origin_sovereign_account = T::SovereignAccountOf::convert_location(origin_location)
				.ok_or(Error::<T>::LocationToSovereignAccountConversionFailed)?;

			T::Token::transfer(
				&origin_sovereign_account,
				&T::TreasuryAccount::get(),
				amount,
				Preservation::Preserve,
			)
		}
	}
}
