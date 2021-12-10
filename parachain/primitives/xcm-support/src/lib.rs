//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use sp_core::U256;
use sp_std::{marker::PhantomData, prelude::*, result};

use xcm::latest::prelude::*;
use xcm_executor::traits::{Convert, TransactAsset};

use snowbridge_core::assets::{
	AssetId as SnowbridgeAssetId, MultiAsset as SnowbridgeMultiAsset, XcmTransactAsset,
};

pub struct AssetsTransactor<Assets, AccountIdConverter, AccountId>(
	PhantomData<(Assets, AccountIdConverter, AccountId)>,
);

impl<
		Assets: SnowbridgeMultiAsset<AccountId>,
		AccountIdConverter: Convert<MultiLocation, AccountId>,
		AccountId: Clone,
	> AssetsTransactor<Assets, AccountIdConverter, AccountId>
{
	fn match_assets(a: &MultiAsset) -> result::Result<(SnowbridgeAssetId, U256), XcmError> {
		let (id, amount) = if let MultiAsset { id, fun: Fungible(amount) } = a {
			(id, amount)
		} else {
			return Err(XcmError::AssetNotFound)
		};

		let key = if let Concrete(location) = id {
			if let Some(GeneralKey(key)) = location.last() {
				key
			} else {
				return Err(XcmError::AssetNotFound)
			}
		} else {
			return Err(XcmError::AssetNotFound)
		};

		let asset_id: SnowbridgeAssetId = SnowbridgeAssetId::decode(&mut key.as_ref())
			.map_err(|_| XcmError::FailedToTransactAsset("AssetIdConversionFailed"))?;

		let value: U256 = (*amount).into();

		Ok((asset_id, value))
	}
}

impl<
		Assets: SnowbridgeMultiAsset<AccountId>,
		AccountIdConverter: Convert<MultiLocation, AccountId>,
		AccountId: Clone,
	> TransactAsset for AssetsTransactor<Assets, AccountIdConverter, AccountId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> XcmResult {
		let (asset_id, amount) = Self::match_assets(asset)?;
		let who = AccountIdConverter::convert_ref(location)
			.map_err(|()| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
		Assets::deposit(asset_id, &who, amount)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
		return Ok(())
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		location: &MultiLocation,
	) -> Result<xcm_executor::Assets, XcmError> {
		let (asset_id, amount) = Self::match_assets(asset)?;
		let who = AccountIdConverter::convert_ref(location)
			.map_err(|()| XcmError::FailedToTransactAsset("AccountIdConversionFailed"))?;
		Assets::withdraw(asset_id, &who, amount)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;
		Ok(asset.clone().into())
	}
}

pub struct XcmAssetTransactor<AccountId>(PhantomData<AccountId>);

impl<AccountId> XcmTransactAsset<AccountId> for XcmAssetTransactor<AccountId> {
	fn reserve_transfer(
		_asset_id: SnowbridgeAssetId,
		_dest: &AccountId,
		_amount: U256,
	) -> frame_support::dispatch::DispatchResult {
		//let origin_location = T::ExecuteXcmOrigin::ensure_origin(origin.clone())?;
		//	use xcm::latest::{
		//		ExecuteXcm,
		//		Instruction::{BuyExecution, DepositAsset, TransferReserveAsset},
		//		MultiAssetFilter::Wild,
		//		MultiLocation,
		//		WildMultiAsset::All,
		//		Xcm,
		//};
		//pub use weights::WeightInfo;
		//use xcm_executor::traits::WeightBounds;
		//type ExecuteXcmOrigin: EnsureOrigin<Self::Origin, Success = MultiLocation>;
		//type XcmExecutor: ExecuteXcm<Self::Call>;
		// Means of measuring the weight consumed by an XCM message locally.
		//type Weigher: WeightBounds<<Self as frame_system::Config>::Call>;
		//let mut message = Xcm(vec![TransferReserveAsset {
		//    assets: todo!(),
		//    dest: todo!(),
		//    xcm: Xcm(vec![
		//        BuyExecution {
		//            fees: todo!(),
		//            weight_limit: todo!(),
		//        },
		//        DepositAsset {
		//            assets: Wild(All),
		//            max_assets: todo!(),
		//            beneficiary: todo!(),
		//        },
		//    ]),
		//}]);
		//let weight = T::Weigher::weight(&mut message)
		//    .map_err(|_| DispatchError::Other("Unweighable message."))?;
		//T::XcmExecutor::execute_xcm(origin_location, message, weight)
		//    .ensure_complete()
		//    .map_err(|_| DispatchError::Other("Xcm execution failed."))?;
		Ok(())
	}
}
