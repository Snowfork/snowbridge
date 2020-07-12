
use frame_support::dispatch::DispatchResult;

/// Selector for target application
///
/// This is an opaque byte string that can only be decoded by verifiers and 
/// target applications.
///
/// For example it could contain the hash of an Ethereum contract address.
pub type AppID = [u8; 32];

/// RLP-encoded message
pub type Message = Vec<u8>;


pub trait Bridge {

    fn deposit_event(app_id: AppID, name: Vec<u8>, data: Vec<u8>);

}

pub trait Broker {

    fn submit(app_id: AppID, message: Message) -> DispatchResult;

}

pub trait Verifier {

    fn verify(app_id: AppID, message: Message) -> DispatchResult;

}

pub trait Application {

    fn is_handler_for(app_id: AppID) -> bool;

    fn handle(app_id: AppID, message: Message) -> DispatchResult;

}