// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Governance API for controlling the Ethereum side of the bridge
//!
//! # Extrinsics
//!
//! ## Agents
//!
//! Agents are smart contracts on Ethereum that act as proxies for consensus systems on Polkadot
//! networks.
//!
//! * [`Call::create_agent`]: Create agent for a sibling parachain
//! * [`Call::transfer_native_from_agent`]: Withdraw ether from an agent
//!
//! The `create_agent` extrinsic should be called via an XCM `Transact` instruction from the sibling
//! parachain.
//!
//! ## Channels
//!
//! Each sibling parachain has its own dedicated messaging channel for sending and receiving
//! messages. As a prerequisite to creating a channel, the sibling should have already created
//! an agent using the `create_agent` extrinsic.
//!
//! * [`Call::create_channel`]: Create channel for a sibling
//! * [`Call::update_channel`]: Update a channel for a sibling
//!
//! ## Governance
//!
//! Only Polkadot governance itself can call these extrinsics. Delivery fees are waived.
//!
//! * [`Call::upgrade`]`: Upgrade the gateway contract
//! * [`Call::set_operating_mode`]: Update the operating mode of the gateway contract
//! * [`Call::force_update_channel`]: Allow root to update a channel for a sibling
//! * [`Call::force_transfer_native_from_agent`]: Allow root to withdraw ether from an agent
//!
//! Typically, Polkadot governance will use the `force_transfer_native_from_agent` and
//! `force_update_channel` and extrinsics to manage agents and channels for system parachains.
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
use sp_io::hashing::blake2_256;
use sp_runtime::{traits::BadOrigin, DispatchError};
use sp_std::prelude::*;
use xcm::prelude::*;
use xcm_executor::traits::ConvertLocation;

use frame_support::{
	pallet_prelude::*,
	traits::{tokens::Preservation, EnsureOrigin},
};
use frame_system::pallet_prelude::*;
use snowbridge_core::{
	outbound::{Command, Initializer, Message, OperatingMode, SendError, SendMessage},
	sibling_sovereign_account, AgentId, Channel, ChannelId, ParaId,
};

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
	/// Fully charge includes (local + remote fee)
	Yes(AccountIdOf<T>),
	/// Partially charge includes local only
	Partial(AccountIdOf<T>),
	/// No charge
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

		/// Send messages to Ethereum
		type OutboundQueue: SendMessage<Balance = BalanceOf<Self>>;

		#[pallet::constant]
		type GovernanceChannelId: Get<ChannelId>;

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
		CreateChannel { channel_id: ChannelId, agent_id: AgentId },
		/// An UpdateChannel message was sent to the Gateway
		UpdateChannel { channel_id: ChannelId, mode: OperatingMode, fee: u128 },
		/// An SetOperatingMode message was sent to the Gateway
		SetOperatingMode { mode: OperatingMode },
		/// An TransferNativeFromAgent message was sent to the Gateway
		TransferNativeFromAgent { agent_id: AgentId, recipient: H160, amount: u128 },
		/// A SetTokenTransferFees message was sent to the Gateway
		SetTokenTransferFees { register: u128, send: u128 },
	}

	#[pallet::error]
	pub enum Error<T> {
		LocationConversionFailed,
		AgentAlreadyCreated,
		NoAgent,
		ChannelAlreadyCreated,
		NoChannel,
		UnsupportedLocationVersion,
		InvalidLocation,
		Send(SendError),
		InvalidTokenTransferFees,
	}

	/// The set of registered agents
	#[pallet::storage]
	#[pallet::getter(fn agents)]
	pub type Agents<T: Config> = StorageMap<_, Twox64Concat, AgentId, (), OptionQuery>;

	/// The set of registered channels
	#[pallet::storage]
	#[pallet::getter(fn channels)]
	pub type Channels<T: Config> = StorageMap<_, Twox64Concat, ChannelId, Channel, OptionQuery>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		// Own parachain id
		pub para_id: ParaId,
		// Own agent id
		pub agent_id: AgentId,
		// AssetHub's parachain id
		pub asset_hub_para_id: ParaId,
		#[serde(skip)]
		pub _config: PhantomData<T>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			let asset_hub_location: MultiLocation =
				ParentThen(X1(Parachain(self.asset_hub_para_id.into()))).into();
			let asset_hub_agent_id =
				agent_id_of::<T>(&asset_hub_location).expect("infallible; qed");
			let asset_hub_channel_id: ChannelId = self.asset_hub_para_id.into();
			Agents::<T>::insert(asset_hub_agent_id, ());
			Channels::<T>::insert(
				asset_hub_channel_id,
				Channel { agent_id: asset_hub_agent_id, para_id: self.asset_hub_para_id },
			);

			Agents::<T>::insert(self.agent_id, ());
			Channels::<T>::insert(
				T::GovernanceChannelId::get(),
				Channel { agent_id: self.agent_id, para_id: self.para_id },
			);
		}
	}

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

			let initializer_params_hash: Option<H256> =
				initializer.as_ref().map(|i| H256::from(blake2_256(i.params.as_ref())));
			let command = Command::Upgrade { impl_address, impl_code_hash, initializer };
			Self::send(T::GovernanceChannelId::get(), command, PaysFee::<T>::No)?;

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
			Self::send(T::GovernanceChannelId::get(), command, pays_fee)?;

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

			let channel_id: ChannelId = para_id.into();

			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::NoAgent);
			ensure!(!Channels::<T>::contains_key(channel_id), Error::<T>::ChannelAlreadyCreated);

			let channel = Channel { agent_id, para_id };
			Channels::<T>::insert(channel_id, channel);

			let command = Command::CreateChannel { channel_id, agent_id };
			let pays_fee = PaysFee::<T>::Yes(sibling_sovereign_account::<T>(para_id));
			Self::send(T::GovernanceChannelId::get(), command, pays_fee)?;

			Self::deposit_event(Event::<T>::CreateChannel { channel_id, agent_id });
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

			let channel_id: ChannelId = para_id.into();

			ensure!(Channels::<T>::contains_key(channel_id), Error::<T>::NoChannel);

			let command = Command::UpdateChannel { channel_id, mode, fee };
			let pays_fee = PaysFee::<T>::Partial(sibling_sovereign_account::<T>(para_id));
			Self::send(channel_id, command, pays_fee)?;

			Self::deposit_event(Event::<T>::UpdateChannel { channel_id, mode, fee });
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

			let channel_id: ChannelId = para_id.into();

			ensure!(Channels::<T>::contains_key(channel_id), Error::<T>::NoChannel);

			let command = Command::UpdateChannel { channel_id, mode, fee };
			Self::send(T::GovernanceChannelId::get(), command, PaysFee::<T>::No)?;

			Self::deposit_event(Event::<T>::UpdateChannel { channel_id, mode, fee });
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
			Self::send(T::GovernanceChannelId::get(), command, PaysFee::<T>::No)?;

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

			Self::do_transfer_native_from_agent(
				agent_id,
				para_id.into(),
				recipient,
				amount,
				pays_fee,
			)
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
			let (_, agent_id) =
				ensure_sibling::<T>(&location).map_err(|_| Error::<T>::InvalidLocation)?;

			let pays_fee = PaysFee::<T>::No;

			Self::do_transfer_native_from_agent(
				agent_id,
				T::GovernanceChannelId::get(),
				recipient,
				amount,
				pays_fee,
			)
		}

		/// Sends a message to the Gateway contract to set token transfer fees
		///
		/// Privileged. Can only be called by root.
		///
		/// Fee required: No
		///
		/// - `origin`: Must be root
		/// - `register`: The fee for register token
		/// - `send`: The fee for send token to parachain
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::set_token_transfer_fees())]
		pub fn set_token_transfer_fees(
			origin: OriginFor<T>,
			register: u128,
			send: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(register > 0 && send > 0, Error::<T>::InvalidTokenTransferFees);

			let command = Command::SetTokenTransferFees { register, send };
			Self::send(T::GovernanceChannelId::get(), command, PaysFee::<T>::No)?;

			Self::deposit_event(Event::<T>::SetTokenTransferFees { register, send });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Send `command` to the Gateway on the Channel identified by `channel_id`
		fn send(channel_id: ChannelId, command: Command, pays_fee: PaysFee<T>) -> DispatchResult {
			let message = Message { id: None, channel_id, command };
			let (ticket, fee) =
				T::OutboundQueue::validate(&message).map_err(|err| Error::<T>::Send(err))?;

			let payment = match pays_fee {
				PaysFee::Yes(account) => Some((account, fee.total())),
				PaysFee::Partial(account) => Some((account, fee.local)),
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

			T::OutboundQueue::deliver(ticket).map_err(|err| Error::<T>::Send(err))?;
			Ok(())
		}

		/// Issue a `Command::TransferNativeFromAgent` command. The command will be sent on the
		/// channel `channel_id`
		pub fn do_transfer_native_from_agent(
			agent_id: H256,
			channel_id: ChannelId,
			recipient: H160,
			amount: u128,
			pays_fee: PaysFee<T>,
		) -> DispatchResult {
			ensure!(Agents::<T>::contains_key(agent_id), Error::<T>::NoAgent);

			let command = Command::TransferNativeFromAgent { agent_id, recipient, amount };
			Self::send(channel_id, command, pays_fee)?;

			Self::deposit_event(Event::<T>::TransferNativeFromAgent {
				agent_id,
				recipient,
				amount,
			});
			Ok(())
		}
	}
}
