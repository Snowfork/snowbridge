// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Governance API for controlling the Ethereum side of the bridge
use super::*;
use frame_support::traits::OnRuntimeUpgrade;
use log;

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod v1 {
	use frame_support::{pallet_prelude::*, weights::Weight};

	use super::*;

	// TODO(alistair): Remove logging
	const LOG_TARGET: &str = "ethereum_system::migration";

	pub struct MigrateToV1<T, BridgeHubParaId, AssetHubParaId>(
		sp_std::marker::PhantomData<(T, BridgeHubParaId, AssetHubParaId)>,
	);
	impl<T, BridgeHubParaId, AssetHubParaId> OnRuntimeUpgrade
		for MigrateToV1<T, BridgeHubParaId, AssetHubParaId>
	where
		T: Config,
		BridgeHubParaId: Get<u32>,
		AssetHubParaId: Get<u32>,
	{
		fn on_runtime_upgrade() -> Weight {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			if onchain_version == 0 && current_version == 1 {
				Pallet::<T>::initialize(
					BridgeHubParaId::get().into(),
					AssetHubParaId::get().into(),
				)
				.expect("infallible; qed");
				log::info!(
					target: LOG_TARGET,
					"Ethereum system initialized. current: {current_version:?} onchain: {onchain_version:?}"
				);
				T::DbWeight::get().reads(1)
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration already applied. This probably can be removed. current: {current_version:?} onchain: {onchain_version:?}"
				);
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 0,
				"must upgrade linearly"
			);
			frame_support::ensure!(
				!Channels::<T>::contains_key(PRIMARY_GOVERNANCE_CHANNEL),
				"primary channel must not exist"
			);
			frame_support::ensure!(
				!Channels::<T>::contains_key(SECONDARY_GOVERNANCE_CHANNEL),
				"secondary channel must not exist"
			);
			log::info!(
				target: LOG_TARGET,
				"Pre upgrade version check successful."
			);
			Ok(vec![])
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), TryRuntimeError> {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();

			frame_support::ensure!(current_version == 1, "must_upgrade");
			ensure!(
				current_version == onchain_version,
				"after migration, the current_version and onchain_version should be the same"
			);
			frame_support::ensure!(
				Channels::<T>::contains_key(PRIMARY_GOVERNANCE_CHANNEL),
				"primary channel must exist"
			);
			frame_support::ensure!(
				Channels::<T>::contains_key(SECONDARY_GOVERNANCE_CHANNEL),
				"secondary channel must exist"
			);
			log::info!(
				target: LOG_TARGET,
				"Post upgrade version check successful."
			);

			Ok(())
		}
	}
}
