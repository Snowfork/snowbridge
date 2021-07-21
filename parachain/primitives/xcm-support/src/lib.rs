//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::U256;
use sp_std::{marker::PhantomData, prelude::*};

use xcm::v0::{Error as XcmError, Junction, MultiAsset, MultiLocation, Result as XcmResult};

use xcm_executor::traits::{Convert, TransactAsset};

use snowbridge_core::assets::{AssetId, MultiAsset as ArtemisMultiAsset};

use codec::Decode;

pub struct AssetsTransactor<Assets, AccountIdConverter, AccountId>(
	PhantomData<(Assets, AccountIdConverter, AccountId)>,
);

impl<
		Assets: ArtemisMultiAsset<AccountId>,
		AccountIdConverter: Convert<MultiLocation, AccountId>,
		AccountId: sp_std::fmt::Debug + Clone,
	> TransactAsset for AssetsTransactor<Assets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		let who = AccountIdConverter::convert_ref(location)
			.map_err(|()| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;

		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			if let Some(Junction::GeneralKey(key)) = id.last() {
				let asset_id: AssetId =
					AssetId::decode(&mut key.as_ref()).map_err(|_| XcmError::Undefined)?;
				let value: U256 = (*amount).into();
				Assets::deposit(asset_id, &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(())
			} else {
				Err(XcmError::Undefined)
			}
		} else {
			Err(XcmError::Undefined)
		}
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		location: &MultiLocation,
	) -> Result<xcm_executor::Assets, XcmError> {
		let who = AccountIdConverter::convert_ref(location)
			.map_err(|()| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;

		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			if let Some(Junction::GeneralKey(key)) = id.last() {
				let asset_id: AssetId =
					AssetId::decode(&mut key.as_ref()).map_err(|_| XcmError::Undefined)?;
				let value: U256 = (*amount).into();
				Assets::withdraw(asset_id, &who, value).map_err(|_| XcmError::Undefined)?;
				Ok(asset.clone().into())
			} else {
				Err(XcmError::Undefined)
			}
		} else {
			Err(XcmError::Undefined)
		}
	}
}
