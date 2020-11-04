#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_std::{
	marker::PhantomData,
	prelude::*,
	if_std,
	convert::TryFrom
};

use frame_support::traits::Currency;

use xcm::v0::{
	Junction,
	MultiAsset,
	MultiLocation,
	Result as XcmResult,
	Error as XcmError,
};

use artemis_core::assets::MultiAsset as ArtemisMultiAsset;

use xcm_executor::traits::{LocationConversion, TransactAsset};


pub trait TransactorConfig {
	type LocalCurrency: Currency<Self::AccountId>;
	type BridgedAssets: ArtemisMultiAsset<Self::AccountId, AssetId = H160>;
	type AccountIdConverter: LocationConversion<Self::AccountId>;
	type AccountId: sp_std::fmt::Debug;
}

pub struct Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId>(
	PhantomData<(LocalCurrency, BridgedAssets, AccountIdConverter, AccountId)>,
);

impl<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId> TransactAsset for Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId>
where
	LocalCurrency: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
	AccountIdConverter: LocationConversion<AccountId>,
	AccountId: sp_std::fmt::Debug
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if let MultiAsset::ConcreteFungible { id, amount } = asset {
			match id {
				MultiLocation::X1(Junction::Parent) => {
					if_std! { println!("Deposit DOT"); }
					let value = <<LocalCurrency as Currency<AccountId>>::Balance as TryFrom<u128>>::try_from(*amount)
						.map_err(|_| ())?;

					let _ = LocalCurrency::deposit_creating(&who, value);
					Ok(())
				},
				MultiLocation::X1(Junction::GeneralIndex { id: 1 }) => {
					if_std! { println!("Deposit ETH"); }
					let bridged_asset_id = H160::zero();
					let value = (*amount).into();
					BridgedAssets::deposit(bridged_asset_id, &who, value).map_err(|_| XcmError::Undefined)
				},
				MultiLocation::X2(
					Junction::GeneralIndex { id: 1 },
					Junction::GeneralKey(key)) => {
					if_std! { println!("Deposit ERC20"); }

					let bridged_asset_id = Self::convert_to_address(key.as_slice()).ok_or(())?;
					let value = (*amount).into();
					BridgedAssets::deposit(bridged_asset_id, &who, value).map_err(|_| XcmError::Undefined)
				},
				_ => {
					Err(XcmError::Undefined)
				}
			}
		} else {
			Err(XcmError::Undefined)
		}
	}

	// withdraw asset from who's account
	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> Result<MultiAsset, XcmError> {
		if_std! {
			println!("WITHDRAW:");
			println!("asset: {:?}", asset);
			println!("location: {:?}", location);
		}

		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if_std! {
			println!("who: {:?}", who);
		}

		if let MultiAsset::ConcreteFungible { id, .. } = asset {
			match id {
				MultiLocation::X1(Junction::Parent) => {
					// Withdraw DOT
					if_std! { println!("withdraw DOT"); }
				},
				MultiLocation::X1(Junction::GeneralIndex { id: 1 }) => {
					// Withdraw ETH
					if_std! { println!("withdraw ETH"); }
				},
				MultiLocation::X2(
					Junction::GeneralIndex { id: 1 },
					Junction::GeneralKey(_key)) => {
					// Withdraw ERC20
					if_std! { println!("withdraw ERC20"); }
				},
				_ => {
					// Handle unknown asset
				}
			}
		}

		Ok(asset.clone())
	}
}

impl<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId> Transactor<LocalCurrency, BridgedAssets, AccountIdConverter, AccountId> {


	fn convert_to_address(slice: &[u8]) -> Option<H160>{
		let mut buf: [u8; 20] = [0; 20];
		if slice.len() != buf.len() {
			return None
		}
		buf.copy_from_slice(slice);
    	Some(buf.into())
	}



}
