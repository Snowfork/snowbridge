//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::U256;
use sp_std::{
	marker::PhantomData,
	prelude::*,
};

use xcm::v0::{
	Junction,
	MultiAsset,
	MultiLocation,
	Result as XcmResult,
	Error as XcmError,
};

use xcm_executor::traits::{LocationConversion, TransactAsset};

use artemis_core::assets::{MultiAsset as ArtemisMultiAsset, AssetId};

use codec::Decode;


pub struct AssetsTransactor<Assets, AccountIdConverter, AccountId>(
	PhantomData<(Assets, AccountIdConverter, AccountId)>,
);

impl<
	Assets: ArtemisMultiAsset<AccountId>,
	AccountIdConverter: LocationConversion<AccountId>,
	AccountId: sp_std::fmt::Debug
	> TransactAsset
	for AssetsTransactor<Assets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		let who = AccountIdConverter::from_location(location).ok_or(())?;

		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			match id {
				MultiLocation::X1(Junction::GeneralKey(key)) => {
					let asset_id: AssetId = AssetId::decode(&mut key.as_ref())
						.map_err(|_| XcmError::Undefined)?;
					let value: U256 = (*amount).into();
					Assets::deposit(asset_id, &who, value).map_err(|_| XcmError::Undefined)?;
					Ok(())
				},
				_ => {
					Err(XcmError::Undefined)
				}
			}
		} else {
			Err(XcmError::Undefined)
		}
	}

	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> Result<MultiAsset, XcmError> {
		let who = AccountIdConverter::from_location(location).ok_or(())?;

		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			match id {
				MultiLocation::X1(Junction::GeneralKey(key)) => {
					let asset_id: AssetId = AssetId::decode(&mut key.as_ref())
						.map_err(|_| XcmError::Undefined)?;
					let value: U256 = (*amount).into();
					Assets::withdraw(asset_id, &who, value).map_err(|_| XcmError::Undefined)?;
					Ok(asset.clone())
				},
				_ => {
					Err(XcmError::Undefined)
				}
			}
		} else {
			Err(XcmError::Undefined)
		}
	}
}
