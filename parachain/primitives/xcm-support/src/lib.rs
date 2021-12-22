//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::OriginFor;
use sp_core::U256;
use sp_runtime::DispatchError;
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::*, result};

use xcm::latest::prelude::*;
use xcm_executor::traits::{Convert, TransactAsset, WeightBounds};

use snowbridge_core::assets::{
	AssetId as SnowbridgeAssetId, MultiAsset as SnowbridgeMultiAsset, XcmReserveTransfer,
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

pub struct XcmAssetTransferer<T>(PhantomData<T>);

impl<T> XcmReserveTransfer<T::AccountId, OriginFor<T>> for XcmAssetTransferer<T>
where
	T: pallet_xcm::Config,
	T::AccountId: AsRef<[u8; 32]>,
{
	fn reserve_transfer(
		origin: <T as frame_system::Config>::Origin,
		asset_id: SnowbridgeAssetId,
		para_id: u32,
		recipient: &T::AccountId,
		amount: U256,
	) -> frame_support::dispatch::DispatchResult {
		let origin_location = T::ExecuteXcmOrigin::ensure_origin(origin.clone())?;

		let amount = u128::try_from(amount).map_err(|e| DispatchError::Other(e))?;

		let mut remote_message = Xcm(vec![
			BuyExecution {
				fees: MultiAsset {
					id: AssetId::Concrete(MultiLocation { parents: 0, interior: Junctions::Here }),
					fun: Fungibility::Fungible(0),
				},
				weight_limit: Limited(0),
			},
			DepositAsset {
				assets: Wild(All),
				max_assets: 2,
				beneficiary: MultiLocation {
					parents: 0,
					interior: Junctions::X1(Junction::AccountId32 {
						network: NetworkId::Any,
						id: recipient.as_ref().clone(),
					}),
				},
			},
		]);

		let remote_weight: u64 = T::Weigher::weight(&mut remote_message)
			.map_err(|_| DispatchError::Other("Unweighable message."))?;

		if let Some(BuyExecution {
			weight_limit: Limited(ref mut limit),
			fees: MultiAsset { fun: Fungibility::Fungible(ref mut fee), .. },
		}) = remote_message.0.get_mut(0)
		{
			*limit = remote_weight;
			*fee = remote_weight.into();
		}

		let mut message = Xcm(vec![TransferReserveAsset {
			assets: MultiAssets::from(vec![
				MultiAsset {
					id: AssetId::Concrete(MultiLocation { parents: 0, interior: Junctions::Here }),
					fun: Fungibility::Fungible(remote_weight.into()),
				},
				MultiAsset {
					id: AssetId::Concrete(MultiLocation {
						parents: 0,
						interior: Junctions::X1(Junction::GeneralKey(asset_id.encode())),
					}),
					fun: Fungibility::Fungible(amount),
				},
			]),
			dest: MultiLocation {
				parents: 1,
				interior: Junctions::X2(
					Junction::Parachain(para_id),
					Junction::GeneralKey(asset_id.encode()),
				),
			},
			xcm: remote_message.into(),
		}]);

		let weight = T::Weigher::weight(&mut message)
			.map_err(|_| DispatchError::Other("Unweighable message."))?;

		T::XcmExecutor::execute_xcm_in_credit(origin_location, message, weight, weight)
			.ensure_complete()
			.map_err(|_| DispatchError::Other("Xcm execution failed."))?;

		Ok(())
	}
}
