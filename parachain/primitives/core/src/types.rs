use sp_std::vec::Vec;

use codec::{Encode, Decode};

// Selector for target application
pub type AppID = [u8; 20];

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct Message {
	pub payload: Vec<u8>,
	pub verification: VerificationInput,
}

#[derive(Debug, PartialEq, Copy, Clone, Encode, Decode)]
pub enum VerificationInput {
	Basic {
		block_number: u64,
		event_index: u32,
	},
	None
}

#[derive(Debug, PartialEq, Clone, Encode, Decode)]
pub struct VerifiedMessage {
	pub payload: Vec<u8>,
}
