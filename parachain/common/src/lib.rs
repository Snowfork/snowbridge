
use frame_support::dispatch::DispatchResult;

/// Selector for target application
///
/// This is an opaque byte identifier that can only be decoded by verifiers and
/// target applications.
///
/// For example it could contain an Ethereum contract address.
pub type AppID = [u8; 32];

/// RLP-encoded message
pub type Message = Vec<u8>;


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
