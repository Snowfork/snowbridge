//! # ERC721
//!
//! An application that implements bridged ERC721 NFT assets.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`nft`] runtime module. When an NFT holder burns
//! the token, a `Transfer` event is emitted. An external relayer will listen for this event
//! and relay it to the other chain.
//!
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchError,
	traits::EnsureOrigin,
	weights::Weight,
};
use frame_support::{pallet_prelude::*, transactional};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_std::prelude::*;

use sp_runtime::traits::StaticLookup;
use sp_core::H160;

use artemis_core::{ChannelId, OutboundRouter};

mod payload;
use payload::OutboundPayload;

#[cfg(test)]
mod mock;

// #[cfg(test)]
// mod tests;


pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	/// Weight functions needed for this pallet.
	pub trait WeightInfo {
		fn burn() -> Weight;
		fn mint() -> Weight;
	}

	impl WeightInfo for () {
		fn burn() -> Weight { 0 }
		fn mint() -> Weight { 0 }
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type OutboundRouter: OutboundRouter<Self::AccountId>;

		type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The submitted payload could not be decoded.
		InvalidPayload,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config>
	{
		/// Burned event: token, sender, recipient
		Burned(H160, T::AccountId, H160),
		/// Minted event: token, sender, recipient
		Minted(H160, H160, T::AccountId),
	}

	/// Address of the peer application on the Ethereum side.
	#[pallet::storage]
	#[pallet::getter(fn address)]
	pub type Address<T: Config> = StorageValue<_, H160, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub _marker: PhantomData<T>,
		pub address: H160,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				_marker: PhantomData::<T>,
				address: H160::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			Address::<T>::try_mutate(|addr| -> Result<H160, DispatchError> {
				*addr = self.address;
				Ok(*addr)
			}).expect("Setting address cannot fail during genesis");
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);


	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Burn an ERC721 token balance
		#[pallet::weight(T::WeightInfo::burn())]
		#[transactional]
		pub fn burn(origin: OriginFor<T>, channel_id: ChannelId, token: H160, recipient: H160) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// TODO: tightly coupled with the nft pallet
			//T::Nft::withdraw(AssetId::Token(token), &who)?;

			let message = OutboundPayload {
				token,
				sender: who.clone(),
				recipient,
			};

			T::OutboundRouter::submit(channel_id, &who, Address::<T>::get(), &message.encode())?;
			Self::deposit_event(Event::<T>::Burned(token, who, recipient));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint())]
		#[transactional]
		pub fn mint(origin: OriginFor<T>, token: H160, sender: H160, recipient: <T::Lookup as StaticLookup>::Source) -> DispatchResultWithPostInfo {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != Address::<T>::get() {
				return Err(DispatchError::BadOrigin.into());
			}

			let recipient = T::Lookup::lookup(recipient)?;
			//T::Assets::deposit(AssetId::Token(token), &recipient)?;
			Self::deposit_event(Event::<T>::Minted(token, sender, recipient));

			Ok(().into())
		}

	}
}
