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

use sp_runtime::traits::{AtLeast32BitUnsigned, StaticLookup};
use sp_core::H160;

use artemis_core::{ChannelId, OutboundRouter, nft::Nft};
use artemis_ethereum::U256;

mod payload;
use payload::OutboundPayload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


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

	pub struct TokenInfo {
		pub erc721_id: H160,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type OutboundRouter: OutboundRouter<Self::AccountId>;

		type CallOrigin: EnsureOrigin<Self::Origin, Success=H160>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The Substrate token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;

		/// The NFT pallet trait
		type Nft: Nft<Self::AccountId, Self::TokenId, ()>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The token is already minted
		TokenAlreadyMinted
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

	/// Store ERC721 (contractAddr, contractTokenId) -> Substrate tokenId mapping
	///
	/// Returns `None` if tokenId not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn tokens_by_erc721)]
	pub type TokensByERC721Id<T: Config> = StorageMap<_, Twox64Concat, (H160, U256), T::TokenId>;

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
		pub fn burn(origin: OriginFor<T>, channel_id: ChannelId, token: T::TokenId, sender: T::AccountId, recipient: H160) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			T::Nft::burn(&sender, token)?;

			// 1. Find the ERC721 token from the token info of this token
			// TODO: delete from TokensByERC721 and from TokensInfo

			// let message = OutboundPayload {
			// 	token,
			// 	sender: who.clone(),
			// 	recipient,
			// };
			// T::OutboundRouter::submit(channel_id, &who, Address::<T>::get(), &message.encode())?;
			// Self::deposit_event(Event::<T>::Burned(erc721_token, who, recipient));

			Ok(().into())
		}

		#[pallet::weight(T::WeightInfo::mint())]
		#[transactional]
		pub fn mint(origin: OriginFor<T>, sender: H160, recipient: <T::Lookup as StaticLookup>::Source, token: H160, token_id: U256, token_uri: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = T::CallOrigin::ensure_origin(origin)?;
			if who != Address::<T>::get() {
				return Err(DispatchError::BadOrigin.into());
			}
			if TokensByERC721Id::<T>::contains_key((token, token_id)) {
				return Err(Error::<T>::TokenAlreadyMinted.into());
			}

			let recipient = T::Lookup::lookup(recipient)?;
			let token_info = TokenInfo{
				erc721_id: token,
			};
			let nft_token_id = T::Nft::mint(&recipient, token_uri, ())?;
			TokensByERC721Id::<T>::insert((token, token_id), nft_token_id);

			// TODO: insert into TokensByERC721 and into TokensInfo (in this pallet or in TokenData - how's that extreacted when burning?)

			Self::deposit_event(Event::<T>::Minted(token, sender, recipient));

			Ok(().into())
		}

	}
}
