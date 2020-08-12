#![cfg_attr(not(feature = "std"), no_std)]
///
/// Implementation for PolkaERC20 token assets
///
use sp_std::prelude::*;
use sp_std::{fmt::Debug, convert::TryInto};
use sp_core::{H160, U256, RuntimeDebug};
use sp_runtime::traits::{
	CheckedAdd, MaybeSerializeDeserialize, Member,  AtLeast32BitUnsigned};
use frame_system::{self as system};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, Parameter,
	dispatch::{DispatchResult, DispatchError},
	storage::StorageDoubleMap
};

use codec::{Decode};

use artemis_core::{AppID, Application, Message};
use artemis_ethereum::{self as ethereum, SignedMessage};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type AssetId = H160;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData {
	pub free: U256
}

pub trait Trait: system::Trait {
	type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as GenericAsset {
		pub TotalIssuance: double_map hasher(blake2_128_concat) AssetId, hasher(blake2_128_concat) T::AccountId => U256;
		pub Account:       double_map hasher(blake2_128_concat) AssetId, hasher(blake2_128_concat) T::AccountId => AccountData;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Burned(AssetId, AccountId, U256),
		Minted(AssetId, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got underflowed after burning.
		TotalBurningUnderflow,
		/// Free balance got underflowed after burning.
		FreeBurningUnderflow,
	}
}

decl_module! {

	pub struct Module<T: Trait<I>, I: Instance> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

	}
}

impl<T: Trait> Module<T> {

}
