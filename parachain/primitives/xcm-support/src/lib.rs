//! # XCMP Support
//!
//! Includes an implementation for the `TransactAsset` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::ensure;
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::DispatchError;
use sp_std::{marker::PhantomData, prelude::*};

use xcm::latest::prelude::*;
use xcm_executor::traits::{WeightBounds};

use snowbridge_core::assets::{RemoteParachain, XcmReserveTransfer};

pub struct XcmAssetTransferer<T>(PhantomData<T>);

impl<T> XcmReserveTransfer<T::AccountId, OriginFor<T>> for XcmAssetTransferer<T>
where
	T: pallet_xcm::Config,
	T::AccountId: AsRef<[u8; 32]>,
{
	fn reserve_transfer(
		asset_id: u128,
		recipient: &T::AccountId,
		amount: u128,
		destination: RemoteParachain,
	) -> frame_support::dispatch::DispatchResult {
		ensure!(
			destination.fee > 0u128,
			DispatchError::Other("Fee must be greater than 0 when parachain id is specified.")
		);

		let origin_location: MultiLocation = MultiLocation {
			parents: 0,
			interior: Junctions::X1(Junction::AccountId32 {
				network: NetworkId::Any,
				id: recipient.as_ref().clone(),
			}),
		};

		let mut message = Xcm(vec![
			WithdrawAsset(
				vec![
					MultiAsset {
						id: Concrete(MultiLocation { parents: 1, interior: Junctions::Here }),
						fun: Fungible(destination.fee),
					},
					MultiAsset {
						id: AssetId::Concrete(MultiLocation {
							parents: 0,
							interior: Junctions::X1(Junction::GeneralIndex(asset_id)),
						}),
						fun: Fungibility::Fungible(amount),
					},
				]
				.into(),
			),
			DepositReserveAsset {
				assets: MultiAssetFilter::Wild(All),
				dest: MultiLocation {
					parents: 1,
					interior: Junctions::X1(Junction::Parachain(destination.para_id)),
				},
				xcm: Xcm(vec![
					BuyExecution {
						fees: MultiAsset {
							id: Concrete(MultiLocation { parents: 1, interior: Junctions::Here }),
							fun: Fungible(destination.fee),
						},
						weight_limit: Unlimited,
					},
					DepositAsset {
						assets: Wild(All),
						max_assets: 2,
						beneficiary: origin_location.clone(),
					},
				]),
				max_assets: 2,
			},
		]);

		let weight = T::Weigher::weight(&mut message)
			.map_err(|_| DispatchError::Other("Unweighable message."))?;

		T::XcmExecutor::execute_xcm_in_credit(origin_location, message, weight, weight)
			.ensure_complete()
			.map_err(|_| DispatchError::Other("Xcm execution failed."))?;

		Ok(())
	}
}
