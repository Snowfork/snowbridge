//! # XCMP Support
//!
//! Includes an implementation for the `XcmReserveTransfer` trait, thus enabling
//! withdrawals and deposits to assets via XCMP message execution.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		ensure, log,
		pallet_prelude::*,
		storage::{with_transaction, TransactionOutcome},
	};
	use frame_system::pallet_prelude::*;
	use snowbridge_xcm_support_primitives::{RemoteParachain, TransferInfo, XcmReserveTransfer};
	use sp_core::{H160, H256};
	use sp_runtime::DispatchError;
	use sp_std::prelude::*;
	use xcm::latest::prelude::*;
	use xcm_executor::traits::WeightBounds;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

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
		ExecutionFailed,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The transfer was successfully sent to the destination
		TransferSent(TransferInfo),
		/// The transfer failed. However assets remain on the parachain.
		TransferFailed { info: TransferInfo, error: DispatchError },
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	impl<T: Config> Pallet<T>
	where
		T: pallet_xcm::Config + Config,
	{
		fn reserve_transfer_unsafe(
			asset_id: u128,
			recipient: H256,
			amount: u128,
			destination: RemoteParachain,
		) -> Result<(), Error<T>> {
			ensure!(destination.fee > 0u128, Error::<T>::ZeroFeeSpecified);

			let origin_location: MultiLocation = MultiLocation {
				parents: 0,
				interior: Junctions::X1(Junction::AccountId32 {
					network: None,
					id: recipient.into(),
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
						DepositAsset { assets: Wild(All), beneficiary: origin_location.clone() },
					]),
				},
			]);

			let weight =
				T::Weigher::weight(&mut message).map_err(|_| Error::<T>::UnweighableMessage)?;

			let hash = message.using_encoded(sp_io::hashing::blake2_256);

			T::XcmExecutor::execute_xcm_in_credit(origin_location, message, hash, weight, weight)
				.ensure_complete()
				.map_err(|err| {
					log::error!("Xcm execution failed. Reason: {:?}", err);
					Error::<T>::ExecutionFailed
				})?;

			Ok(())
		}
	}

	impl<T> XcmReserveTransfer<T::AccountId, OriginFor<T>> for Pallet<T>
	where
		T: pallet_xcm::Config + Config,
		T::AccountId: AsRef<[u8; 32]>,
	{
		fn reserve_transfer(
			asset_id: u128,
			sender: H160,
			recipient: &T::AccountId,
			amount: u128,
			destination: RemoteParachain,
		) {
			let recipient: H256 = recipient.as_ref().into();

			let result = with_transaction(|| {
				let outcome =
					Self::reserve_transfer_unsafe(asset_id, recipient, amount, destination);
				match outcome {
					Ok(()) => TransactionOutcome::Commit(Ok(())),
					Err(error) => TransactionOutcome::Rollback(Err(DispatchError::from(error))),
				}
			});

			let info = TransferInfo {
				asset_id,
				sender,
				recipient,
				amount,
				para_id: destination.para_id,
				fee: destination.fee,
			};
			let event = match result {
				Ok(()) => Event::<T>::TransferSent(info),
				Err(error) => Event::<T>::TransferFailed { info, error },
			};
			Self::deposit_event(event);
		}
	}
}
