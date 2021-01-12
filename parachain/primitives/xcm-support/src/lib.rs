//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::U256;
use sp_std::{
	marker::PhantomData,
	prelude::*,
	convert::{TryFrom, TryInto}
};

use frame_support::traits::{Get, Currency, WithdrawReasons, ExistenceRequirement};

use xcm::v0::{
	Junction,
	MultiAsset,
	MultiLocation,
	Result as XcmResult,
	Error as XcmError,
};

use xcm_executor::traits::{NativeAsset, LocationConversion, FilterAssetLocation, TransactAsset};

use artemis_core::assets::{MultiAsset as ArtemisMultiAsset, AssetId};


pub struct Transactor<DOT, BridgedAssets, AccountIdConverter, AccountId>(
	PhantomData<(DOT, BridgedAssets, AccountIdConverter, AccountId)>,
);

impl<
	DOT: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId>,
	AccountIdConverter: LocationConversion<AccountId>,
	AccountId: sp_std::fmt::Debug
	> TransactAsset
	for Transactor<DOT, BridgedAssets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			Self::deposit(id, &who, *amount)
		} else {
			Err(XcmError::Undefined)
		}
	}

	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> Result<MultiAsset, XcmError> {
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			Self::withdraw(id, &who, *amount).map(|_| asset.clone())
		} else {
			Err(XcmError::Undefined)
		}
	}
}


impl<
       DOT: Currency<AccountId>,
       BridgedAssets: ArtemisMultiAsset<AccountId>,
       AccountIdConverter: LocationConversion<AccountId>,
       AccountId: sp_std::fmt::Debug
       > Transactor<DOT, BridgedAssets, AccountIdConverter, AccountId>
{
	fn deposit(id: &MultiLocation, who: &AccountId, amount: u128) -> XcmResult {
		match id {
			// Deposit DOT
			MultiLocation::X1(Junction::Parent) => {
				let value = <<DOT as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
					.map_err(|_| ())?;

				let _ = DOT::deposit_creating(&who, value);
				Ok(())
			},
			// Deposit ETH
			MultiLocation::X2(
				Junction::PalletInstance { id: 0 },
				Junction::GeneralIndex { id: 0 },
			) => {
				let value: U256 = amount.into();
				BridgedAssets::deposit(AssetId::ETH, &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			// Deposit ERC20
			MultiLocation::X3(
				Junction::PalletInstance { id: 0 },
				Junction::GeneralIndex { id: 1 },
				Junction::GeneralKey(key)
			) => {
				let value: U256 = amount.into();
				let key_fixed: [u8; 20] = key.clone().try_into().map_err(|_| XcmError::Undefined)?;
				BridgedAssets::deposit(AssetId::Token(key_fixed.into()), &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			_ => {
				Err(XcmError::Undefined)
			}
		}
	}

	fn withdraw(id: &MultiLocation, who: &AccountId, amount: u128) -> XcmResult {
		match id {
			// Withdraw DOT
			MultiLocation::X1(Junction::Parent) => {
				let value = <<DOT as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(amount)
					.map_err(|_| ())?;

				let _ = DOT::withdraw(&who, value, WithdrawReasons::TRANSFER, ExistenceRequirement::KeepAlive)
					.map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			// Withdraw ETH
			MultiLocation::X2(
				Junction::PalletInstance { id: 0 },
				Junction::GeneralIndex { id: 0 },
			) => {
				let value: U256 = amount.into();
				BridgedAssets::withdraw(AssetId::ETH, &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			// Deposit ERC20
			MultiLocation::X3(
				Junction::PalletInstance { id: 0 },
				Junction::GeneralIndex { id: 1 },
				Junction::GeneralKey(key)
			) => {
				let value: U256 = amount.into();
				let key_fixed: [u8; 20] = key.clone().try_into().map_err(|_| XcmError::Undefined)?;
				BridgedAssets::withdraw(AssetId::Token(key_fixed.into()), &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			_ => {
				Err(XcmError::Undefined)
			}
		}
	}

}

pub struct TrustedReserveFilter<T>(PhantomData<T>);

impl<T: Get<MultiLocation>> FilterAssetLocation for TrustedReserveFilter<T> {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		if NativeAsset::filter_asset_location(asset, origin) {
			return true;
		}

		if let MultiAsset::ConcreteFungible { ref id, .. } = asset {
			match id {
				MultiLocation::X2(
					Junction::PalletInstance { id: 0 },
					Junction::GeneralIndex { id: 0 }
				) => {
						return *origin == T::get()
				},
				MultiLocation::X3(
					Junction::PalletInstance { id: 0 },
					Junction::GeneralIndex { id: 1 },
					Junction::GeneralKey(_)
				) => {
						return *origin == T::get()
				},
				_ => {
					return false
				}
			}
		}
		false
	}
}
