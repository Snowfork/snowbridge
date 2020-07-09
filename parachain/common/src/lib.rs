
use frame_support::dispatch::DispatchResult;

pub type AppID = u64;
pub type Message = Vec<u8>;


pub trait Verifier {

    fn verify(app_id: AppID, message: Message) -> DispatchResult;

}


pub trait Application {

    fn handle(message: Message) -> DispatchResult;

}
