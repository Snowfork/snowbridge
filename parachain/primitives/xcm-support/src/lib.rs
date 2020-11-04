#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_std::{
	marker::PhantomData,
	prelude::*,
	if_std,
};

use frame_support::traits::Currency;

use xcm::v0::{
	Error, Junction,
	MultiAsset,
	MultiLocation,
	Result as XcmResult,
};

use artemis_core::assets::MultiAsset as ArtemisMultiAsset;

use xcm_executor::traits::{LocationConversion, TransactAsset};

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
	// deposit asset into who's account
	// Need to use AccountConverter helpers to convert MultiLocation into a AccountId
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		if_std! {
			println!("DEPOSIT:");
			println!("asset: {:?}", asset);
			println!("location: {:?}", location);
		}

		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if_std! {
			println!("who: {:?}", who);
		}

		if let MultiAsset::ConcreteFungible { id, .. } = asset {
			match id {
				MultiLocation::X1(Junction::Parent) => {},
				MultiLocation::X1(Junction::PalletInstance { id: 2 }) => {},
				_ => {}
			}
		}
		Ok(())

	}

	// withdraw asset from who's account
	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> Result<MultiAsset, Error> {
		if_std! {
			println!("WITHDRAW:");
			println!("asset: {:?}", asset);
			println!("location: {:?}", location);
		}

		let who = AccountIdConverter::from_location(location).ok_or(())?;
		if_std! {
			println!("who: {:?}", who);
		}

		Ok(asset.clone())
	}
}
