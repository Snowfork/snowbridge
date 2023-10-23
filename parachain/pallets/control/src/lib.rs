// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Governance API for controlling the Ethereum side of the bridge
//!
//! * upgrade: Upgrade the gateway contract (permissioned)
//! * set_operating_mode: Update the operating mode of the gateway contract (permissioned)
//! * create_agent: Create agent for a sibling (permissionless)
//! * create_channel: Create channel for a sibling (permissionless)
//! * update_channel: Update a channel for a sibling (permissionless)
//! * force_update_channel: Allow root to update a channel for a sibling (permissioned)
//! * transfer_native_from_agent: Withdraw ether from an agent (permissionless)
//! * force_transfer_native_from_agent: Allow root to withdraw ether from an agent (permissionless)
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod api;
pub mod weights;
pub use weights::*;

use frame_support::traits::fungible::{Inspect, Mutate};
use sp_core::{RuntimeDebug, H160, H256};
use sp_runtime::{
	traits::{BadOrigin, Hash},
	DispatchError, SaturatedConversion,
};
use sp_std::prelude::*;
use xcm::prelude::*;
use xcm_executor::traits::ConvertLocation;

use frame_support::{
	pallet_prelude::*,
	traits::{tokens::Preservation, EnsureOrigin},
};
use frame_system::pallet_prelude::*;
use snowbridge_core::{
	outbound::{
		Command, Initializer, Message, OperatingMode, OutboundQueue as OutboundQueueTrait, ParaId,
	},
	sibling_sovereign_account, AgentId,
};
use sp_runtime::Saturating;

#[cfg(feature = "runtime-benchmarks")]
use frame_support::traits::OriginTrait;

pub use pallet::*;

pub type BalanceOf<T> =
	<<T as pallet::Config>::Token as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Ensure origin location is a sibling
fn ensure_sibling<T>(location: &MultiLocation) -> Result<(ParaId, H256), DispatchError>
where
	T: Config,
{
	match location {
		MultiLocation { parents: 1, interior: X1(Parachain(para_id)) } => {
			let agent_id = agent_id_of::<T>(location)?;
			Ok(((*para_id).into(), agent_id))
		},
		_ => Err(BadOrigin.into()),
	}
}

/// Hash the location to produce an agent id
fn agent_id_of<T: Config>(location: &MultiLocation) -> Result<H256, DispatchError> {
	T::AgentIdOf::convert_location(location).ok_or(Error::<T>::LocationConversionFailed.into())
}

#[cfg(feature = "runtime-benchmarks")]
pub trait BenchmarkHelper<O>
where
	O: OriginTrait,
{
	fn make_xcm_origin(location: MultiLocation) -> O;
}

/// Whether a fee should be withdrawn to an account for sending an outbound message
#[derive(Clone, PartialEq, RuntimeDebug)]
pub enum PaysFee<T>
where
	T: Config,
{
	Yes(AccountIdOf<T>),
	Partial(AccountIdOf<T>),
	No,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// General-purpose hasher
		type MessageHasher: Hash<Output = H256>;

		/// Send messages to Ethereum
		type OutboundQueue: OutboundQueueTrait<Balance = BalanceOf<Self>>;

		/// The ID of this parachain
		type OwnParaId: Get<ParaId>;

		/// Origin check for XCM locations that can create agents
		type SiblingOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;

		/// Converts MultiLocation to AgentId
		type AgentIdOf: ConvertLocation<AgentId>;

		/// Token reserved for control operations
		type Token: Mutate<Self::AccountId>;

		/// TreasuryAccount to collect fees
		#[pallet::constant]
		type TreasuryAccount: Get<Self::AccountId>;

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type Helper: BenchmarkHelper<Self::RuntimeOrigin>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An Upgrade message was sent to the Gateway
		Upgrade { impl_address: H160, impl_code_hash: H256, initializer_params_hash: Option<H256> },
		/// An CreateAgent message was sent to the Gateway
		CreateAgent { location: Box<MultiLocation>, agent_id: AgentId },
		/// An CreateChannel message was sent to the Gateway
		CreateChannel { para_id: ParaId, agent_id: AgentId },
		/// An UpdateChannel message was sent to the Gateway
		UpdateChannel { para_id: ParaId, mode: OperatingMode, fee: u128 },
		/// An SetOperatingMode message was sent to the Gateway
		SetOperatingMode { mode: OperatingMode },
		/// An TransferNativeFromAgent message was sent to the Gateway
		TransferNativeFromAgent { agent_id: AgentId, recipient: H160, amount: u128 },
		/// Redeem from treasury to channel origin
		RedeemFromTreasury { para_id: ParaId, recipient: AccountIdOf<T>, amount: BalanceOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		SubmissionFailed,
		LocationConversionFailed,
		AgentAlreadyCreated,
		NoAgent,
		ChannelAlreadyCreated,
		NoChannel,
		UnsupportedLocationVersion,
		InvalidLocation,
	}

	/// The set of registered agents
	#[pallet::storage]
	#[pallet::getter(fn agents)]
	pub type Agents<T: Config> = StorageMap<_, Twox64Concat, AgentId, (), OptionQuery>;

	/// The set of registered channels
	#[pallet::storage]
	#[pallet::getter(fn channels)]
	pub type Channels<T: Config> = StorageMap<_, Twox64Concat, ParaId, (), OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sends command to the Gateway contract to upgrade itself with a new implementation
		/// contract
		///
		/// Fee required: No
		///
		/// - `origin`: Must be `Root`.
		/// - `impl_address`: The address of the implementation contract.
		/// - `impl_code_hash`: The codehash of the implementation contract.
		/// - `initializer`: Optionally call an initializer on the implementation contract.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::upgrade())]
		pub fn upgrade(
			origin: OriginFor<T>,
			impl_address: H160,
			impl_code_hash: H256,
			initializer: Option<Initializer>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let initializer_params_hash =
				initializer.as_ref().map(|i| T::MessageHasher::hash(i.params.as_ref()));
			let command = Command::Upgrade { impl_address, impl_code_hash, initializer };
			Self::send(T::OwnParaId::get(), command, PaysFee::<T>::No)?;

			Self::deposit_event(Event::<T>::Upgrade {
				impl_address,
				impl_code_hash,
				initializer_params_hash,
			});
			Ok(())
		}

		/// Sends a command to the Gateway contract to instantiate a new agent contract representing
		/// `origin`.
		///
		/// Fee required: Yes
		///
		/// - `origin`: Must be `MultiLocation` of a sibling parachain
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_agent())]
		pub fn create_agent(origin: OriginFor<T>) -> DispatchResult {
			let origin_location: MultiLocation = T::SiblingOrigin::ensure_origin(origin)?;

			// Ensure that origin location is some consensus system on a sibling parachain
			let (para_id, agent_id) = ensure_sibling::<T>(&origin_location)?;

			// Record the agent id or fail if it has already been created
			ensure!(!Agents::<T>::contains_key(agent_id), Error::<T>::AgentAlreadyCreated);
			Agents::<T>::insert(agent_id, ());

			let command = Command::CreateAgent { agent_id };
			let pays_fee = PaysFee::<T>::Yes(sibling_sovereign_account::<T>(para_id));
			Self::send(T::OwnParaId::get(), command, pays_fee)?;

			Self::deposit_event(Event::<T>::CreateAgent {
				location: Box::new(origin_location),
				agent_id,
			});
			Ok(())
		}

		/// Sends a message to the Gateway contract to create a new channel representing `origin`
		///
		/// Fee required: Yes
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
			let origin_location: MultiLocation = T::SiblingOrigin::ensure_origin(origin)?;

			// Ensure that origin location is a sibling parachain
			let (para_id, agent_id) = ensure_sibling::<T>(&origin_location)?;

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::NoAgent);
			ensure!(!Channels::<T>::contains_key(para_id), Error::<T>::ChannelAlreadyCreated);

			Channels::<T>::insert(para_id, ());

			let command = Command::CreateChannel { para_id, agent_id };
			let pays_fee = PaysFee::<T>::Yes(sibling_sovereign_account::<T>(para_id));
			Self::send(T::OwnParaId::get(), command, pays_fee)?;

			Self::deposit_event(Event::<T>::CreateChannel { para_id, agent_id });
			Ok(())
		}

		/// Sends a message to the Gateway contract to update a channel configuration
		///
		/// The origin must already have a channel initialized, as this message is sent over it.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_channel())]
		pub fn update_channel(
			origin: OriginFor<T>,
			mode: OperatingMode,
			fee: u128,
		) -> DispatchResult {
			let origin_location: MultiLocation = T::SiblingOrigin::ensure_origin(origin)?;

			// Ensure that origin location is a sibling parachain
			let (para_id, _) = ensure_sibling::<T>(&origin_location)?;

			ensure!(Channels::<T>::contains_key(para_id), Error::<T>::NoChannel);

			let command = Command::UpdateChannel { para_id, mode, fee };
			let pays_fee = PaysFee::<T>::Partial(sibling_sovereign_account::<T>(para_id));
			Self::send(para_id, command, pays_fee)?;

			Self::deposit_event(Event::<T>::UpdateChannel { para_id, mode, fee });
			Ok(())
		}

		/// Sends a message to the Gateway contract to update a channel configuration
		///
		/// The origin must already have a channel initialized, as this message is sent over it.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be root
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::force_update_channel())]
		pub fn force_update_channel(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			mode: OperatingMode,
			fee: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure that location is a sibling parachain
			let location: MultiLocation =
				(*location).try_into().map_err(|_| Error::<T>::UnsupportedLocationVersion)?;
			let (para_id, _) =
				ensure_sibling::<T>(&location).map_err(|_| Error::<T>::InvalidLocation)?;

			ensure!(Channels::<T>::contains_key(para_id), Error::<T>::NoChannel);

			let command = Command::UpdateChannel { para_id, mode, fee };
			Self::send(para_id, command, PaysFee::<T>::No)?;

			Self::deposit_event(Event::<T>::UpdateChannel { para_id, mode, fee });
			Ok(())
		}

		/// Sends a message to the Gateway contract to change its operating mode
		///
		/// Fee required: No
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::set_operating_mode())]
		pub fn set_operating_mode(origin: OriginFor<T>, mode: OperatingMode) -> DispatchResult {
			ensure_root(origin)?;

			let command = Command::SetOperatingMode { mode };
			Self::send(T::OwnParaId::get(), command, PaysFee::<T>::No)?;

			Self::deposit_event(Event::<T>::SetOperatingMode { mode });
			Ok(())
		}

		/// Sends a message to the Gateway contract to transfer ether from an agent to `recipient`.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer_native_from_agent())]
		pub fn transfer_native_from_agent(
			origin: OriginFor<T>,
			recipient: H160,
			amount: u128,
		) -> DispatchResult {
			let origin_location: MultiLocation = T::SiblingOrigin::ensure_origin(origin)?;

			// Ensure that origin location is some consensus system on a sibling parachain
			let (para_id, agent_id) = ensure_sibling::<T>(&origin_location)?;

			let pays_fee = PaysFee::<T>::Partial(sibling_sovereign_account::<T>(para_id));

			Self::do_transfer_native_from_agent(agent_id, para_id, recipient, amount, pays_fee)
		}

		/// Sends a message to the Gateway contract to transfer ether from an agent to `recipient`.
		///
		/// Privileged. Can only be called by root.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be root
		/// - `location`: Location used to resolve the agent
		/// - `recipient`: Recipient of funds
		/// - `amount`: Amount to transfer
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::force_transfer_native_from_agent())]
		pub fn force_transfer_native_from_agent(
			origin: OriginFor<T>,
			location: Box<VersionedMultiLocation>,
			recipient: H160,
			amount: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			// Ensure that location is some consensus system on a sibling parachain
			let location: MultiLocation =
				(*location).try_into().map_err(|_| Error::<T>::UnsupportedLocationVersion)?;
			let (para_id, agent_id) =
				ensure_sibling::<T>(&location).map_err(|_| Error::<T>::InvalidLocation)?;

			let pays_fee = PaysFee::<T>::No;

			Self::do_transfer_native_from_agent(agent_id, para_id, recipient, amount, pays_fee)
		}

		/// Sends a message to the Gateway contract to transfer ether from an agent to `recipient`.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be `MultiLocation`
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::redeem())]
		pub fn redeem(origin: OriginFor<T>, recipient: AccountIdOf<T>) -> DispatchResult {
			let origin_location: MultiLocation = T::SiblingOrigin::ensure_origin(origin)?;

			// Ensure that origin location is some consensus system on a sibling parachain
			let (para_id, _) = ensure_sibling::<T>(&origin_location)?;

			Self::do_redeem(para_id, recipient)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Send `command` to the Gateway on the channel identified by `origin`.
		fn send(origin: ParaId, command: Command, pays_fee: PaysFee<T>) -> DispatchResult {
			let message = Message { origin, command };
			let (ticket, fee) =
				T::OutboundQueue::validate(&message).map_err(|_| Error::<T>::SubmissionFailed)?;

			let payment = match pays_fee {
				PaysFee::Yes(account) =>
					Some((account, fee.base_fee.saturating_add(fee.delivery_fee))),
				PaysFee::Partial(account) => Some((account, fee.base_fee)),
				PaysFee::No => None,
			};

			if let Some((payer, fee)) = payment {
				T::Token::transfer(
					&payer,
					&T::TreasuryAccount::get(),
					fee,
					Preservation::Preserve,
				)?;
			}

			T::OutboundQueue::submit(ticket).map_err(|_| Error::<T>::SubmissionFailed)?;
			Ok(())
		}

		/// Issue a `Command::TransferNativeFromAgent` command. The command will be sent on the
		/// channel owned by `para_id`.
		pub fn do_transfer_native_from_agent(
			agent_id: H256,
			para_id: ParaId,
			recipient: H160,
			amount: u128,
			pays_fee: PaysFee<T>,
		) -> DispatchResult {
			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::NoAgent);

			let command = Command::TransferNativeFromAgent { agent_id, recipient, amount };
			Self::send(para_id, command, pays_fee)?;

			Self::deposit_event(Event::<T>::TransferNativeFromAgent {
				agent_id,
				recipient,
				amount,
			});
			Ok(())
		}

		/// Issue a `Command::TransferNativeFromAgent` command. The command will be sent on the
		/// channel owned by `para_id`.
		pub fn do_redeem(para_id: ParaId, recipient: T::AccountId) -> DispatchResult {
			ensure!(Channels::<T>::contains_key(para_id), Error::<T>::NoChannel);
			T::OutboundQueue::redeem(para_id, |amount| -> DispatchResult {
				let redeem_amount = (*amount).saturated_into::<u128>().saturated_into();
				T::Token::transfer(
					&T::TreasuryAccount::get(),
					&recipient,
					redeem_amount,
					Preservation::Preserve,
				)?;
				Self::deposit_event(Event::<T>::RedeemFromTreasury {
					para_id,
					recipient,
					amount: redeem_amount,
				});
				Ok(())
			})?;
			Ok(())
		}
	}
}
