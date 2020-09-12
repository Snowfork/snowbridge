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


// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use hex_literal::hex;

// 	#[test]
// 	fn test_encode() {

// 		let msg = Message {
// 			payload: [0, 1, 2].to_vec(),
// 			verification: VerificationInput::Basic { block_number: }
// 		}

// 	}
// }
