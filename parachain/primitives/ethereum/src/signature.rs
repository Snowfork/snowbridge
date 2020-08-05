use secp256k1::{self, Message, Signature, PublicKey};
use sp_io::hashing::keccak_256;

pub fn verify(message: &[u8], public_key: &[u8], signature: &[u8]) -> Result<bool, secp256k1::Error> {
	let hash = keccak_256(message);
	let m = Message::parse(&hash);
	let p = PublicKey::parse_slice(public_key, None)?;
	let s = Signature::parse_slice(&signature[0..64])?;
	return Ok(secp256k1::verify(&m, &s, &p));
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex::FromHex;

	const PUBLIC_KEY: &'static str = concat!(
		"43ee8c1f93b3df3f6af36ca73270797d6feca8a16e290cffedc8f53d66d6150027772f574a5804d2",
		"b4b66245610f506bf4652e7f89c817b5083aad44b1f79676"
	);

	const MESSAGE: &'static str = concat!(
		"f9013a94f60d0eab5b5e800c89357048bfc8d3d30a2bcd93e1a06bafbf13bfcea5e4ce5cd1a03b24",
		"6069acefcd0bada5ef4e1a059b37a08c2399b9010000000000000000000000000000000000000000",
		"00000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"000000004000000000000000000000000000000000000000000000000000000000000000a0000000",
		"000000000000000000cffeaaf7681c89285d65cfbe808b80e5026965738eaf04151687736326c9fe",
		"a17e25fc5287613693c912909cb226aa4794f26a4800000000000000000000000000000000000000",
		"00000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"000000000a0000000000000000000000000000000000000000000000000000000000000018"
	);

	const SIGNATURE: &'static str = concat!(
		"dabdeebd6739b7be5d5b9f4641e4d8f20ddeac0f7020dd2bfce3f4a25e258d8d122fa4fc164868e1",
		"0b82f4af91bffda31a58bce3309931a18f9160c6c2a6bb3700"
	);

	#[test]
	fn it_verifies_message() {

		let public_key: Vec<u8> = PUBLIC_KEY.from_hex().unwrap();
		let message: Vec<u8> = MESSAGE.from_hex().unwrap();
		let signature: Vec<u8> = SIGNATURE.from_hex().unwrap();

		assert_eq!(verify(&message, &public_key, &signature).unwrap(), true);
	}

}