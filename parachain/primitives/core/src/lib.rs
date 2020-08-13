#![allow(dead_code)]
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

//use sp_std::prelude::*;
use frame_support::dispatch::DispatchResult;
use codec::Encode;

pub mod types;
pub mod registry;

pub use types::{AppID, Message};


/// The bridge module implements this trait
pub trait TransferEventEmitter<K> where K: Encode {

	fn emit(app_id: &AppID, data: K);

}

/// The broker module implements this trait
pub trait Broker {

	fn submit(app_id: AppID, message: Message) -> DispatchResult;

}

/// The verifier module implements this trait
pub trait Verifier {

	fn verify(app_id: AppID, message: Message) -> DispatchResult;

}

/// The dummy app module implements this trait
pub trait Application {

	/// Handle a message
	fn handle(app_id: AppID, message: Message) -> DispatchResult;

}
