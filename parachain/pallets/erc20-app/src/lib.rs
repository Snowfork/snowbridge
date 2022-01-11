//! # ERC20
//!
//! An application that implements bridged ERC20 token assets.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`asset`] runtime module. When an account holder
//! burns some of their balance, a `Transfer` event is emitted. An external relayer will listen for
//! this event and relay it to the other chain.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ERC20 token balance.
#![cfg_attr(not(feature = "std"), no_std)]

mod payload;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	traits::tokens::fungibles::{Create, Mutate},
	traits::{EnsureOrigin, Randomness},
	transactional, PalletId,
};
use frame_system::ensure_signed;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{AccountIdConversion, Hash, StaticLookup, TrailingZeroInput},
	TokenError,
};
use sp_std::prelude::*;

use snowbridge_core::{assets::XcmReserveTransfer, ChannelId, OutboundRouter};

use payload::OutboundPayload;
pub use weights::WeightInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		type Hashing: Hash<Output = H256>;

		type Assets: Create<Self::AccountId, Balance = u128, AssetId = u128>
			+ Mutate<Self::AccountId, Balance = u128, AssetId = u128>;

		type OutboundRouter: OutboundRouter<Self::AccountId>;

		type CallOrigin: EnsureOrigin<Self::Origin, Success = H160>;

		type WeightInfo: WeightInfo;

		type XcmReserveTransfer: XcmReserveTransfer<Self::AccountId, Self::Origin>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Burned(H160, T::AccountId, H160, u128),
		Minted(H160, H160, T::AccountId, u128),
	}

	#[pallet::storage]
	#[pallet::getter(fn address)]
	pub(super) type Address<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn asset_id)]
	pub(super) type AssetId<T: Config> = StorageMap<_, Identity, H160, u128, OptionQuery>;

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub address: H160,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { address: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<Address<T>>::put(self.address);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({
			match channel_id {
				ChannelId::Basic => T::WeightInfo::burn_basic_channel(),
				ChannelId::Incentivized => T::WeightInfo::burn_incentivized_channel(),
			}
		})]
		#[transactional]
		pub fn burn(
			origin: OriginFor<T>,
			channel_id: ChannelId,
			token: H160,
			recipient: H160,
			amount: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let asset_id =
				Self::asset_id(token).ok_or(DispatchError::Token(TokenError::UnknownAsset))?;

			T::Assets::burn_from(asset_id, &who, amount)?;

			let message = OutboundPayload {
				token,
				sender: who.clone(),
				recipient: recipient.clone(),
				amount,
			};

			T::OutboundRouter::submit(channel_id, &who, <Address<T>>::get(), &message.encode())?;
			Self::deposit_event(Event::Burned(token, who.clone(), recipient, amount));

			Ok(())
		}

		#[pallet::weight(T::WeightInfo::mint())]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			token: H160,
			sender: H160,
			recipient: <T::Lookup as StaticLookup>::Source,
			amount: u128,
			para_id: Option<u32>,
		) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin.clone())?;
			if who != <Address<T>>::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let asset_id =
				Self::asset_id(token).ok_or(DispatchError::Token(TokenError::UnknownAsset))?;

			let recipient = T::Lookup::lookup(recipient)?;

			T::Assets::mint_into(asset_id, &recipient, amount)?;

			if let Some(id) = para_id {
				T::XcmReserveTransfer::reserve_transfer(origin, asset_id, id, &recipient, amount)?;
			}

			Self::deposit_event(Event::Minted(token, sender, recipient, amount));

			Ok(())
		}

		#[pallet::weight(100_000_000)]
		#[transactional]
		pub fn create(origin: OriginFor<T>, token: H160) -> DispatchResult {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != <Address<T>>::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let asset_id = Self::make_asset_id(token);
			T::Assets::create(asset_id, T::PalletId::get().into_account(), true, 1)?;

			<AssetId<T>>::insert(token, asset_id);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn make_asset_id(address: H160) -> u128 {
			let (seed, _) = T::Randomness::random(b"erc20app");

			let mut input = [0u8; 52];

			// seed should be at least 32 bytes
			let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
				.expect("input is padded with zeroes; qed");

			input[..32].copy_from_slice(&seed);
			input[32..].copy_from_slice(address.as_fixed_bytes().as_ref());

			let hash = <T as Config>::Hashing::hash(&input);
			let mut output = [0u8; 16];
			output.copy_from_slice(&hash.as_fixed_bytes()[..16]);

			u128::from_le_bytes(output)
		}
	}
}
