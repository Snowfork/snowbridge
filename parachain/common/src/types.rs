use sp_std::prelude::*;
use codec::{Encode, Decode};

/// Selector for target application
///
/// This is an opaque byte identifier that can only be decoded by verifiers and
/// target applications.
///
/// For example it could contain an Ethereum contract address.
pub type AppID = [u8; 32];

/// Raw message from relayer
pub type Message = Vec<u8>;

/// Signed message decoded from raw message
#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SignedMessage {
	pub data: Vec<u8>,
	pub signature: Vec<u8>
}

#[cfg(test)]
mod tests {

	use std::io::prelude::*;
	use std::fs::File;
	use std::io::BufReader;
	use std::path::PathBuf;

	use super::*;

	fn fixture_path() -> PathBuf {
		[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", "packet.scale"].iter().collect()
	}

	#[test]
	fn decode_packet() {
		let mut reader = BufReader::new(File::open(fixture_path()).unwrap());
		let mut data: Vec<u8> = Vec::new();
		reader.read_to_end(&mut data).unwrap();

		assert_eq!(Packet::decode(&mut data.as_slice()).is_ok(), true);
	}


}
