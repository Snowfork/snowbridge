#![allow(dead_code)]
#![allow(unused_variables)]

use frame_support::dispatch::DispatchResult;

pub mod types;
pub mod registry;

pub use types::{AppID, Message, MessageHash};


/// The bridge module implements this trait
pub trait Bridge {

    fn deposit_event(app_id: AppID, name: Vec<u8>, data: Vec<u8>);

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

    /// Checks whether this app is a handler for messages submitted with `app_id`.
    fn is_handler_for(app_id: AppID) -> bool;

    /// Handle a message
    fn handle(app_id: AppID, message: Message) -> DispatchResult;

}
