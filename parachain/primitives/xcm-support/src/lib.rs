//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{H160, U256};
use sp_std::{
	marker::PhantomData,
	prelude::*,
	convert::TryFrom
};

use frame_support::traits::{Currency, WithdrawReasons, ExistenceRequirement};

use xcm::v0::{
	Junction,
	MultiAsset,
	MultiLocation,
	NetworkId,
	Result as XcmResult,
	Error as XcmError,
};

use artemis_core::assets::MultiAsset as ArtemisMultiAsset;
use xcm_executor::traits::{LocationConversion, TransactAsset};

pub struct Transactor<DOT, BridgedAssets, AccountIdConverter, AccountId>(
	PhantomData<(DOT, BridgedAssets, AccountIdConverter, AccountId)>,
);

impl<
	DOT: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
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
       BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
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
			// Deposit ETH, ERC20
			MultiLocation::X2(
				Junction::PalletInstance { id: 11 },
				Junction::AccountKey20 { network: NetworkId::Any, key }) => {
				let value: U256 = amount.into();
				BridgedAssets::deposit(key.into(), &who, value).map_err(|_| XcmError::Undefined)?;
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

				let _ = DOT::withdraw(&who, value, WithdrawReasons::none(), ExistenceRequirement::KeepAlive)
					.map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			// Withdraw ETH, ERC20
			MultiLocation::X2(
				Junction::PalletInstance { id: 11 },
				Junction::AccountKey20 { network: NetworkId::Any, key }) => {
				let value: U256 = amount.into();
				BridgedAssets::withdraw(key.into(), &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			},
			_ => {
				Err(XcmError::Undefined)
			}
		}
	}

}
