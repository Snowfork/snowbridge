//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::traits::EnsureOrigin;
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::DispatchError;
use sp_std::{marker::PhantomData, prelude::*};

use xcm::latest::prelude::*;
use xcm_executor::traits::{WeightBounds};

use snowbridge_core::assets::XcmReserveTransfer;

pub struct XcmAssetTransferer<T>(PhantomData<T>);

impl<T> XcmReserveTransfer<T::AccountId, OriginFor<T>> for XcmAssetTransferer<T>
where
	T: pallet_xcm::Config,
	T::AccountId: AsRef<[u8; 32]>,
{
	fn reserve_transfer(
		origin: <T as frame_system::Config>::Origin,
		asset_id: u128,
		para_id: u32,
		recipient: &T::AccountId,
		amount: u128,
	) -> frame_support::dispatch::DispatchResult {
		let origin_location = T::ExecuteXcmOrigin::ensure_origin(origin.clone())?;

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
