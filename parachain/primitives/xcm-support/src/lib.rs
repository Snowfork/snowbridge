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

use xcm_executor::traits::TransactAsset;

pub struct Transactor<LocalCurrency, BridgedAssets, AccountId>(
	PhantomData<(LocalCurrency, BridgedAssets, AccountId)>,
);

impl<LocalCurrency, BridgedAssets, AccountId> TransactAsset for Transactor<LocalCurrency, BridgedAssets, AccountId>
where
	LocalCurrency: Currency<AccountId>,
	BridgedAssets: ArtemisMultiAsset<AccountId, AssetId = H160>,
	AccountId: sp_std::fmt::Debug
{
	// deposit asset into who's account
	// Need to use AccountConverter helpers to convert MultiLocation into a AccountId
	fn deposit_asset(asset: &MultiAsset, _who: &MultiLocation) -> XcmResult {
		if_std! {
			println!("DEPOSIT:");
			println!("asset: {:?}", asset);
			println!("who: {:?}", _who);
		}

		if let MultiAsset::ConcreteFungible { id, .. } = asset {
			match id {
				MultiLocation::X1(Junction::Parent) => {},
				MultiLocation::X3(
					Junction::Parent,
					Junction::Parachain { .. },
					Junction::PalletInstance { id: 4 }) => {},
				_ => {}
			}
		}
		Ok(())
	}

	// withdraw asset from who's account
	fn withdraw_asset(asset: &MultiAsset, _who: &MultiLocation) -> Result<MultiAsset, Error> {
		if_std! {
			println!("WITHDRAW:");
			println!("asset: {:?}", asset);
			println!("who: {:?}", _who);
		}

		Ok(asset.clone())
	}
}
