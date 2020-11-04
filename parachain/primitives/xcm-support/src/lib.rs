#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{H160, U256};
use sp_std::{
	marker::PhantomData,
	prelude::*,
	if_std,
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


pub struct Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId>(
	PhantomData<(LocalCurrency, BridgedAssets, AccountIdConverter, AccountId)>,
);

impl<
	LocalCurrency: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
	AccountIdConverter: LocationConversion<AccountId>,
	AccountId: sp_std::fmt::Debug
	> TransactAsset
	for Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		if_std!{ println!("{:?}", location) }
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			match id {
				MultiLocation::X1(Junction::PalletInstance { id: 2 }) => {
					let value = <<LocalCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(*amount)
						.map_err(|_| ())?;

					let _ = LocalCurrency::deposit_creating(&who, value);
					Ok(())
				},
				MultiLocation::X2(
					Junction::PalletInstance { id: 11 },
					Junction::AccountKey20 { network: NetworkId::Any, key }) => {
					let value: U256 = (*amount).into();
					Self::deposit_bridged_asset(key.into(), &who, value)
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
		if_std!{ println!("{:?}", location) }

		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			match id {
				MultiLocation::X1(Junction::PalletInstance { id: 2 }) => {
					let value = <<LocalCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(*amount)
						.map_err(|_| ())?;

					let _ = LocalCurrency::withdraw(&who, value, WithdrawReasons::none(), ExistenceRequirement::KeepAlive)
						.map_err(|_| XcmError::Undefined)?;
					Ok(asset.clone())
				},
				MultiLocation::X2(
					Junction::PalletInstance { id: 11 },
					Junction::AccountKey20 { network: NetworkId::Any, key }) => {
					let value: U256 = (*amount).into();
					Self::withdraw_bridged_asset(key.into(), &who, value)?;
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

impl<
	LocalCurrency: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
	AccountIdConverter: LocationConversion<AccountId>,
	AccountId: sp_std::fmt::Debug
	> Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId>
{
	fn deposit_bridged_asset(asset_id: H160, who: &AccountId, amount: U256) -> XcmResult {
		BridgedAssets::deposit(asset_id, who, amount).map_err(|_| XcmError::Undefined)
	}

	fn withdraw_bridged_asset(asset_id: H160, who: &AccountId, amount: U256) -> XcmResult {
		BridgedAssets::withdraw(asset_id, who, amount).map_err(|_| XcmError::Undefined)
	}
}
