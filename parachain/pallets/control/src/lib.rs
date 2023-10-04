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
		Command, Message, OperatingMode, OutboundQueue as OutboundQueueTrait, ParaId, Initializer
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
	/// The location of this origin, reanchored to be relative to the relay chain
	pub reanchored_location: MultiLocation,
	/// The parachain hosting this origin
	pub para_id: ParaId,
	/// The deterministic ID of the agent for this origin
	pub agent_id: H256,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
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

		/// Permissionless operations require an upfront fee to prevent spamming
		type Fee: Get<BalanceOf<Self>>;

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
			mode: OperatingMode,
			fee: u128,
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
		LocationConversionFailed,
		AgentAlreadyCreated,
		AgentNotExist,
		ChannelAlreadyCreated,
		ChannelNotExist,
		LocationToAccountConversionFailed,
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
		#[pallet::weight(T::WeightInfo::upgrade())]
		pub fn upgrade(
			origin: OriginFor<T>,
			impl_address: H160,
			impl_code_hash: H256,
			initializer: Option<Initializer>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let params_hash = initializer.as_ref().map(|i| T::MessageHasher::hash(i.params.as_ref()));

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::Upgrade { impl_address, impl_code_hash, initializer },
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
			let origin_location: MultiLocation = T::AgentOrigin::ensure_origin(origin)?;

			Self::charge_fee(&origin_location)?;

			let OriginInfo { reanchored_location, agent_id, .. } =
				Self::process_origin_location(&origin_location)?;

			// Record the agent id or fail if it has already been created
			ensure!(!Agents::<T>::contains_key(agent_id), Error::<T>::AgentAlreadyCreated);

			Agents::<T>::insert(agent_id, ());

			let message =
				Message { origin: T::OwnParaId::get(), command: Command::CreateAgent { agent_id } };
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::CreateAgent { location: Box::new(reanchored_location), agent_id });
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new Channel representing `origin`
		///
		/// This extrinsic is permissionless, so a fee is charged to prevent spamming and pay
		/// for execution costs on the remote side.
		///
		/// The message is sent over the bridge on BridgeHub's own channel to the Gateway.
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_channel())]
		pub fn create_channel(origin: OriginFor<T>) -> DispatchResult {
			let origin_location: MultiLocation = T::ChannelOrigin::ensure_origin(origin)?;

			Self::charge_fee(&origin_location)?;

			let OriginInfo { para_id, agent_id, .. } =
				Self::process_origin_location(&origin_location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);
			ensure!(!Channels::<T>::contains_key(para_id), Error::<T>::ChannelAlreadyCreated);

			Channels::<T>::insert(para_id, ());

			let message = Message {
				origin: T::OwnParaId::get(),
				command: Command::CreateChannel { agent_id, para_id },
			};
			Self::submit_outbound(message)?;
			Self::deposit_event(Event::<T>::CreateChannel { para_id, agent_id });

			Ok(())
		}

		/// Sends a message to the Gateway contract to update a channel configuration
		///
		/// The origin must already have a channel initialized, as this message is sent over it.
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_channel())]
		pub fn update_channel(
			origin: OriginFor<T>,
			mode: OperatingMode,
			fee: u128,
		) -> DispatchResult {
			let origin_location: MultiLocation = T::ChannelOrigin::ensure_origin(origin)?;

			let OriginInfo { agent_id, para_id, .. } =
				Self::process_origin_location(&origin_location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);
			ensure!(Channels::<T>::contains_key(para_id), Error::<T>::ChannelNotExist);

			let message = Message {
				origin: para_id,
				command: Command::UpdateChannel { para_id, mode, fee },
			};
			Self::submit_outbound(message)?;
			Self::deposit_event(Event::<T>::UpdateChannel { para_id, mode, fee });

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
			Self::submit_outbound(message)?;

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
			let origin_location: MultiLocation = T::AgentOrigin::ensure_origin(origin)?;

			let OriginInfo { agent_id, para_id, .. } =
				Self::process_origin_location(&origin_location)?;

				Self::do_transfer_native_from_agent(agent_id, para_id, recipient, amount)
		}

		/// Sends a message to the Gateway contract to transfer asset from an an agent.
		///
		/// Privileged. Can only be called by root.
		///
		/// - `origin`: Must be `MultiLocation`
		/// - `location`: Location used to resolve the agent
		/// - `recipient`: Recipient of funds
		/// - `amount`: Amount to transfer
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer_native_from_agent())]
		pub fn force_transfer_native_from_agent(
				origin: OriginFor<T>,
				location: Box<VersionedMultiLocation>,
				recipient: H160,
				amount: u128,
		) -> DispatchResult {
			ensure_root(origin)?;
			let location = *location;
			let location: MultiLocation = location.try_into().map_err(|_| Error::<T>::LocationConversionFailed)?;
			let OriginInfo { agent_id, .. } =
				Self::process_origin_location(&location)?;

			Self::do_transfer_native_from_agent(agent_id, T::OwnParaId::get(), recipient, amount)
		}
	}

	impl<T: Config> Pallet<T> {
		fn submit_outbound(message: Message) -> DispatchResult {
			let (ticket, _) =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;
			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;
			Ok(())
		}

		// Normalize origin locations relative to the relay chain.
		pub fn reanchor_origin_location(location: &MultiLocation) -> Result<MultiLocation, DispatchError> {
			let relay_location = T::RelayLocation::get();

			let mut reanchored_location = *location;
			reanchored_location
				.reanchor(&relay_location, T::UniversalLocation::get())
				.map_err(|_| Error::<T>::LocationReanchorFailed)?;

			Ok(reanchored_location)
		}

		pub fn process_origin_location(
			location: &MultiLocation,
		) -> Result<OriginInfo, DispatchError> {
			let reanchored_location = Self::reanchor_origin_location(location)?;

			let para_id = match reanchored_location.interior.first() {
				Some(Parachain(index)) => Some((*index).into()),
				_ => None,
			}
			.ok_or(Error::<T>::LocationConversionFailed)?;

			// Hash the location to produce an agent id
			let agent_id = T::AgentIdOf::convert_location(&reanchored_location)
				.ok_or(Error::<T>::LocationConversionFailed)?;

			Ok(OriginInfo { reanchored_location, para_id, agent_id, })
		}

		/// Charge a flat fee from the sovereign account of the origin location
		fn charge_fee(origin_location: &MultiLocation) -> DispatchResult {
			let sovereign_account = T::SovereignAccountOf::convert_location(origin_location)
				.ok_or(Error::<T>::LocationToAccountConversionFailed)?;

			T::Token::transfer(
				&sovereign_account,
				&T::TreasuryAccount::get(),
				T::Fee::get(),
				Preservation::Preserve,
			)?;

			Ok(())
		}

		pub fn do_transfer_native_from_agent(agent_id: H256, para_id: ParaId, recipient: H160, amount: u128) -> DispatchResult {
			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::AgentNotExist);

			let message = Message {
				origin: para_id,
				command: Command::TransferNativeFromAgent { agent_id, recipient, amount },
			};
			Self::submit_outbound(message)?;

			Self::deposit_event(Event::<T>::TransferNativeFromAgent {
				agent_id,
				recipient,
				amount,
			});

			Ok(())
		}
	}
}
