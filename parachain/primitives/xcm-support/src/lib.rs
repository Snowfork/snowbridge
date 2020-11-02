#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::H160;
use sp_std::{
	marker::PhantomData,
	prelude::*,
};

use frame_support::traits::Currency;
use frame_support::dispatch::DispatchResult;

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
	fn deposit_asset(asset: &MultiAsset, _who: &MultiLocation) -> XcmResult {
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

	fn withdraw_asset(asset: &MultiAsset, _who: &MultiLocation) -> Result<MultiAsset, Error> {
		Ok(asset.clone())
	}
}

pub trait XcmHandler {
	type Origin;
	type Xcm;
	fn execute(origin: Self::Origin, xcm: Self::Xcm) -> DispatchResult;
}
