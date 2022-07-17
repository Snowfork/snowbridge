//! # XCMP Support
//!
//! Includes an implementation for the `XcmReserveTransfer` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{ensure, log, pallet_prelude::*, storage::{with_transaction, TransactionOutcome}};
	use frame_system::pallet_prelude::*;
	use snowbridge_xcm_support_primitives::{RemoteParachain, XcmReserveTransfer};
	use sp_runtime::DispatchError;
	use sp_std::prelude::*;
	use xcm::latest::prelude::*;
	use xcm_executor::traits::WeightBounds;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig {}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Fee must be greater than zero.
		ZeroFeeSpecified,
		/// Message was not able to be weighed.
		UnweighableMessage,
		/// Xcm execution failed during initiation of request.
		XcmExecutionFailed
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T> XcmReserveTransfer<T::AccountId, OriginFor<T>> for Pallet<T>
	where
		T: pallet_xcm::Config + Config,
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
				DispatchError::from(Error::<T>::ZeroFeeSpecified)
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
								id: Concrete(MultiLocation {
									parents: 1,
									interior: Junctions::Here,
								}),
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
				.map_err(|_| DispatchError::from(Error::<T>::UnweighableMessage))?;

			let _ = with_transaction(|| { 
				let outcome = T::XcmExecutor::execute_xcm_in_credit(origin_location, message, weight, weight)
					.ensure_complete()
					.map_err(|err| {
						log::error!("Xcm execution failed. Reason: {:?}", err);
						DispatchError::from(Error::<T>::XcmExecutionFailed)
					});

				match outcome {
					Ok(()) => TransactionOutcome::Commit(outcome),
					Err(_) => TransactionOutcome::Rollback(outcome)
				}
			});

			Ok(())
		}
	}
}
