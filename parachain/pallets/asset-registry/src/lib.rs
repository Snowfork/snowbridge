#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_runtime::ArithmeticError;

	#[pallet::config]
	pub trait Config: frame_system::Config {
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn next_asset_id)]
	pub type NextAssetId<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub next_asset_id: u128,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self { next_asset_id: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			<NextAssetId<T>>::put(self.next_asset_id);
		}
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	impl<T: Config> snowbridge_asset_registry_primitives::NextAssetId for Pallet<T> {
		fn next() -> Result<u128, DispatchError> {
			<NextAssetId<T>>::try_mutate(|value| {
				let id = *value;
				*value = value.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				Ok(id)
			})
		}
	}
}
