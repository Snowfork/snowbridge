//! # Non Fungible Token
//! The module provides implementations for non-fungible-token.
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The NFT module is used by the Polkadot-Ethereum bridge to store ERC721 tokens.

//! ### Module Functions
//!
//! - `transfer` - Transfer NFT(non fungible token) to another account.
//! - `mint` - Mint NFT(non fungible token)
//! - `burn` - Burn NFT(non fungible token)

#![cfg_attr(not(feature = "std"), no_std)]


use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, Parameter};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, MaybeSerializeDeserialize, Member, One},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;

// mod mock;
// mod tests;

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<AccountId, Data> {
	/// Token owner
	pub owner: AccountId,
	/// Token metadata
	pub metadata: Vec<u8>,
	/// Token Properties
	pub data: Data,
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
		/// The token properties type
		type TokenData: Parameter + Member + MaybeSerializeDeserialize;
	}

	pub type TokenInfoOf<T> = TokenInfo<<T as frame_system::Config>::AccountId, <T as Config>::TokenData>;

	pub type GenesisTokenData<T> = (
		<T as frame_system::Config>::AccountId, // Token owner
		Vec<u8>,                                // Token metadata
		<T as Config>::TokenData,               // Token data
	);

	/// Error for non-fungible-token module.
	#[pallet::error]
	pub enum Error<T> {
		/// No available token ID
		NoAvailableTokenId,
		/// Token(ClassId, TokenId) not found
		TokenNotFound,
		/// The operator is not the owner of the token and has no permission
		NoPermission,
		/// Arithmetic calculation overflow
		NumOverflow,
	}

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_token_id)]
	pub type NextTokenId<T: Config> = StorageValue<_, T::TokenId, ValueQuery>;

	/// Store token info.
	///
	/// Returns `None` if token info not set or removed.
	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	pub type Tokens<T: Config> = StorageMap<_, Twox64Concat, T::TokenId, TokenInfoOf<T>>;

	/// Token existence check by owner
	#[pallet::storage]
	#[pallet::getter(fn tokens_by_owner)]
	pub type TokensByOwner<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::TokenId, (), ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub tokens: Vec<GenesisTokenData<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { tokens: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.tokens.iter().for_each(|(owner, metadata, data)| {
				Pallet::<T>::mint(&owner.clone(), metadata.clone(), data.clone())
					.expect("Token mint cannot fail during genesis");
			})
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	/// Transfer NFT(non fungible token) from `from` account to `to` account
	pub fn transfer(from: &T::AccountId, to: &T::AccountId, token: T::TokenId) -> DispatchResult {
		Tokens::<T>::try_mutate(token, |token_info| -> DispatchResult {
			let mut info = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(info.owner == *from, Error::<T>::NoPermission);
			if from == to {
				// no change needed
				return Ok(());
			}

			info.owner = to.clone();
			TokensByOwner::<T>::remove(from, token);
			TokensByOwner::<T>::insert(to, token, ());

			Ok(())
		})
	}

	/// Mint NFT(non fungible token) to `owner`
	pub fn mint(
		owner: &T::AccountId,
		metadata: Vec<u8>,
		data: T::TokenData,
	) -> Result<T::TokenId, DispatchError> {
		NextTokenId::<T>::try_mutate(|id| -> Result<T::TokenId, DispatchError> {
			let token_id = *id;
			*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;

			let token_info = TokenInfo {
				metadata,
				owner: owner.clone(),
				data,
			};
			Tokens::<T>::insert(token_id, token_info);
			TokensByOwner::<T>::insert(owner, token_id, ());

			Ok(token_id)
		})
	}

	/// Burn NFT(non fungible token) from `owner`
	pub fn burn(owner: &T::AccountId, token: T::TokenId) -> DispatchResult {
		Tokens::<T>::try_mutate_exists(token, |token_info| -> DispatchResult {
			let t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
			ensure!(t.owner == *owner, Error::<T>::NoPermission);

			TokensByOwner::<T>::remove(owner, token);

			Ok(())
		})
	}
}
