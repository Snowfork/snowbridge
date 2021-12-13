//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::OriginFor;
use sp_core::U256;
use sp_runtime::DispatchError;
use sp_std::{marker::PhantomData, prelude::*, result};

use xcm::latest::prelude::*;
use xcm_executor::traits::{Convert, TransactAsset, WeightBounds};

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

pub struct XcmAssetTransactor<T, XcmExecutor, ExecuteXcmOrigin, Weigher>(
	PhantomData<(T, XcmExecutor, ExecuteXcmOrigin, Weigher)>,
);

impl<T, XcmExecutor, ExecuteXcmOrigin, Weigher> XcmTransactAsset<T::AccountId, OriginFor<T>>
	for XcmAssetTransactor<T, XcmExecutor, ExecuteXcmOrigin, Weigher>
where
	T: eth_app::Config + erc20_app::Config,
	XcmExecutor: ExecuteXcm<T::Call>,
	ExecuteXcmOrigin: EnsureOrigin<T::Origin, Success = MultiLocation>,
	Weigher: WeightBounds<<T as frame_system::Config>::Call>,
{
	fn reserve_transfer(
		origin: T::Origin,
		_asset_id: SnowbridgeAssetId,
		_dest: &T::AccountId,
		_amount: U256,
	) -> frame_support::dispatch::DispatchResult {
		//pub use weights::WeightInfo;
		//use xcm_executor::traits::WeightBounds;
		//	use xcm::latest::{
		//		ExecuteXcm,
		//		Instruction::{BuyExecution, DepositAsset, TransferReserveAsset},
		//		MultiAssetFilter::Wild,
		//		MultiLocation,
		//		WildMultiAsset::All,
		//		Xcm,
		//};
		//type Weigher: WeightBounds<<Self as frame_system::Config>::Call>;

		let origin_location = ExecuteXcmOrigin::ensure_origin(origin.clone())?;
		// Means of measuring the weight consumed by an XCM message locally.
		let mut message = Xcm(vec![TransferReserveAsset {
			assets: todo!(),
			dest: todo!(),
			xcm: Xcm(vec![
				BuyExecution { fees: todo!(), weight_limit: todo!() },
				DepositAsset { assets: Wild(All), max_assets: todo!(), beneficiary: todo!() },
			]),
		}]);
		let weight = Weigher::weight(&mut message)
			.map_err(|_| DispatchError::Other("Unweighable message."))?;
		XcmExecutor::execute_xcm(origin_location, message, weight)
			.ensure_complete()
			.map_err(|_| DispatchError::Other("Xcm execution failed."))?;
		Ok(())
	}
}
