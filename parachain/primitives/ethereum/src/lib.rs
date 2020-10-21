#![cfg_attr(not(feature = "std"), no_std)]

use ethbloom::Bloom as EthBloom;
use parity_bytes::Bytes;
use rlp::RlpStream;
use sp_io::hashing::keccak_256;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use serde_big_array::big_array;

pub mod log;

pub use ethereum_types::{Address, H160, H256, U256};
pub use log::Log;

#[derive(Debug)]
pub enum DecodeError {
	// Unexpected RLP data
	InvalidRLP(rlp::DecoderError),
	// Data does not match expected ABI
	InvalidABI(ethabi::Error),
	// Invalid message payload
	InvalidPayload,
}

impl From<rlp::DecoderError> for DecodeError {
	fn from(err: rlp::DecoderError) -> Self {
		DecodeError::InvalidRLP(err)
	}
}

impl From<ethabi::Error> for DecodeError {
	fn from(err: ethabi::Error) -> Self {
		DecodeError::InvalidABI(err)
	}
}

/// Complete block header id.
#[derive(Clone, Copy, Default, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct HeaderId {
	/// Header number.
	pub number: u64,
	/// Header hash.
	pub hash: H256,
}

/// An Ethereum block header.
#[derive(Clone, Default, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Header {
	/// Parent block hash.
	pub parent_hash: H256,
	/// Block timestamp.
	pub timestamp: u64,
	/// Block number.
	pub number: u64,
	/// Block author.
	pub author: Address,

	/// Transactions root.
	pub transactions_root: H256,
	/// Block ommers hash.
	pub ommers_hash: H256,
	/// Block extra data.
	pub extra_data: Bytes,

	/// State root.
	pub state_root: H256,
	/// Block receipts root.
	pub receipts_root: H256,
	/// Block bloom.
	pub logs_bloom: Bloom,
	/// Gas used for contracts execution.
	pub gas_used: U256,
	/// Block gas limit.
	pub gas_limit: U256,

	/// Block difficulty.
	pub difficulty: U256,
	/// Vector of post-RLP-encoded fields.
	pub seal: Vec<Bytes>,
}

impl Header {
	/// Compute hash of this header (keccak of the RLP with seal).
	pub fn compute_hash(&self) -> H256 {
		keccak_256(&self.rlp(true)).into()
	}

	/// Returns header RLP with or without seals.
	fn rlp(&self, with_seal: bool) -> Bytes {
		let mut s = RlpStream::new();
		if with_seal {
			s.begin_list(13 + self.seal.len());
		} else {
			s.begin_list(13);
		}

		s.append(&self.parent_hash);
		s.append(&self.ommers_hash);
		s.append(&self.author);
		s.append(&self.state_root);
		s.append(&self.transactions_root);
		s.append(&self.receipts_root);
		s.append(&EthBloom::from(self.logs_bloom.0));
		s.append(&self.difficulty);
		s.append(&self.number);
		s.append(&self.gas_limit);
		s.append(&self.gas_used);
		s.append(&self.timestamp);
		s.append(&self.extra_data);

		if with_seal {
			for b in &self.seal {
				s.append_raw(b, 1);
			}
		}

		s.out()
	}
}

#[cfg(feature = "std")]
big_array! { BigArray; }

/// Logs bloom.
#[derive(Clone, Debug, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Bloom(#[cfg_attr(feature = "std", serde(with = "BigArray"))] [u8; 256]);

impl<'a> From<&'a [u8; 256]> for Bloom {
	fn from(buffer: &'a [u8; 256]) -> Bloom {
		Bloom(*buffer)
	}
}

impl PartialEq<Bloom> for Bloom {
	fn eq(&self, other: &Bloom) -> bool {
		self.0.iter().zip(other.0.iter()).all(|(l, r)| l == r)
	}
}

impl Default for Bloom {
	fn default() -> Self {
		Bloom([0; 256])
	}
}


#[cfg(test)]
mod tests {

	use super::*;
	use hex_literal::hex;

	#[test]
	fn header_compute_hash_poa() {
		// PoA header
		let header = Header {
			parent_hash: Default::default(),
			timestamp: 0,
			number: 0,
			author: Default::default(),
			transactions_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
			ommers_hash: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
			extra_data: vec![],
			state_root: hex!("eccf6b74c2bcbe115c71116a23fe963c54406010c244d9650526028ad3e32cce").into(),
			receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
			logs_bloom: Default::default(),
			gas_used: Default::default(),
			gas_limit: 0x222222.into(),
			difficulty: 0x20000.into(),
			seal: vec![vec![0x80], {
				let mut vec = vec![0xb8, 0x41];
				vec.resize(67, 0);
				vec
			}],
		};
		assert_eq!(
			header.compute_hash().as_bytes(),
			hex!("9ff57c7fa155853586382022f0982b71c51fa313a0942f8c456300896643e890"),
		);
	}

	#[test]
	fn header_compute_hash_pow() {
		// https://etherscan.io/block/11090290
		let nonce = hex!("6935bbe7b63c4f8e").to_vec();
		let mix_hash = hex!("be3adfb0087be62b28b716e2cdf3c79329df5caa04c9eee035d35b5d52102815").to_vec();
		let header = Header {
			parent_hash: hex!("bede0bddd6f32c895fc505ffe0c39d9bde58e9a5272f31a3dee448b796edcbe3").into(),
			timestamp: 1603160977,
			number: 11090290,
			author: hex!("ea674fdde714fd979de3edf0f56aa9716b898ec8").into(),
			transactions_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
			ommers_hash: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
			extra_data: hex!("65746865726d696e652d61736961312d33").to_vec(),
			state_root: hex!("7dcb8aca872b712bad81df34a89d4efedc293566ffc3eeeb5cbcafcc703e42c9").into(),
			receipts_root: hex!("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421").into(),
			logs_bloom: Default::default(),
			gas_used: 0.into(),
			gas_limit: 0xbe8c19.into(),
			difficulty: 0xbc140caa61087i64.into(),
			seal: vec![
				rlp::encode(&mix_hash),
				rlp::encode(&nonce),
			],
		};
		assert_eq!(
			header.compute_hash().as_bytes(),
			hex!("0f9bdc91c2e0140acb873330742bda8c8181fa3add91fe7ae046251679cedef7"),
		);
	}

}
