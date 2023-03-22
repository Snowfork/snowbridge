#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchResult},
};


use snowbridge_router_primitives::{Action, NativeTokensAction};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::event]
	pub enum Event<T: Config> {

	}

	impl<T: Config> Pallet<T> {
		pub fn handle_action(action: &Action) -> DispatchResult {
			match action {
				Action::NativeTokens(subaction) => {
					match subaction {
						NativeTokensAction::Create { .. } => {
							Ok(())
						},
						_ => {
							Ok(())
						}
					}					
				},
			}
		}
	}

}
