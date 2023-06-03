#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

use frame_support::traits::EnsureOrigin;
use snowbridge_core::{OutboundQueue as OutboundQueueTrait, ParaId};
use sp_core::{H160, H256};
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;
use xcm::v3::MultiLocation;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type MessageHasher: Hash<Output = H256>;
		type OutboundQueue: OutboundQueueTrait;
		type RemarkOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = H160>;
		type HandleRemarkOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = MultiLocation>;
		type OwnParaId: Get<ParaId>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Remarked { sender: H160, hash: H256 },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		// #[pallet::call_index(0)]
		// #[pallet::weight(T::WeightInfo::remark())]
		// pub fn upgrade(origin: OriginFor<T>, upgrade_task: H160) -> DispatchResult {
		// 	let who = ensure_root(origin)?;
		// 	let message_id = Self::make_message_id(who, upgrade_task.as_ref());
		// 	let _ = T::OutboundQueue::submit(message_id, T::OwnParaId::get(), 2, &_remark);
		// 	Ok(())
		// }
	}

	impl<T: Config> Pallet<T>
	where
		T::AccountId: AsRef<[u8]>,
	{
		fn make_message_id(who: T::AccountId, remark: &[u8]) -> H256 {
			let who: Vec<u8> = who.as_ref().into();
			let appended: Vec<u8> = who.iter().copied().chain(remark.iter().copied()).collect();
			T::MessageHasher::hash(&appended)
		}
	}
}
