use sp_std::vec::Vec;
use codec::{Encode, Decode};
use hex::FromHex;

use crate::signature;

// NOTE: For this current milestone, we're using a hardcoded public key
pub const PUBLIC_KEY: &str = concat!(
	"43ee8c1f93b3df3f6af36ca73270797d6feca8a16e290cffedc8f53d66d6150027772f574a5804d2",
	"b4b66245610f506bf4652e7f89c817b5083aad44b1f79676"
);

/// Signed message decoded from raw message
#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SignedMessage {
	pub data: Vec<u8>,
	pub signature: Vec<u8>
}

impl SignedMessage {

	pub fn verify(&self) -> Result<bool, secp256k1::Error> {
		let public_key: Vec<u8> = match PUBLIC_KEY.from_hex() {
			Ok(key) => key,
			Err(_) => return Err(secp256k1::Error::InvalidPublicKey)
		};
		signature::verify(&self.data, &public_key, &self.signature)
	}

}


#[cfg(test)]
mod tests {

	use std::io::prelude::*;
	use std::fs::File;
	use std::io::BufReader;
	use std::path::PathBuf;
	use hex::ToHex;

	use super::*;

	fn fixture_path() -> PathBuf {
		[env!("CARGO_MANIFEST_DIR"), "tests", "fixtures", "packet.scale"].iter().collect()
	}

	#[test]
	fn decode_packet() {
		let mut reader = BufReader::new(File::open(fixture_path()).unwrap());
		let mut data: Vec<u8> = Vec::new();
		reader.read_to_end(&mut data).unwrap();

		assert_eq!(SignedMessage::decode(&mut data.as_slice()).is_ok(), true);

		let sm = SignedMessage::decode(&mut data.as_slice()).unwrap();
		println!("{:?}", sm.data.as_slice().to_hex::<String>());
		println!("{:?}", sm.signature.as_slice().to_hex::<String>());
	}


}
