use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use codec::Encode;
use sp_core::H256;
use sp_runtime::{offchain::storage::StorageValueRef, traits::Keccak256};

use snowbridge_basic_channel_merkle_proof::merkle_proof;

pub struct BasicChannel;
impl BasicChannel {
	pub fn new() -> Self {
		Self {}
	}
}

#[rpc]
pub trait BasicChannelApi {
	#[rpc(name = "basicChannel_outbound_getMerkleProof")]
	fn get_merkle_proof(&self, commitment_hash: H256, leaf_index: u64) -> Result<Vec<u8>>;
}

impl BasicChannelApi for BasicChannel {
	fn get_merkle_proof(&self, commitment_hash: H256, leaf_index: u64) -> Result<Vec<u8>> {
		let oci_mem = StorageValueRef::persistent(&commitment_hash.as_bytes());

		if let Ok(Some(leaves)) = oci_mem.get::<Vec<Vec<u8>>>() {
			let proof =
				merkle_proof::<Keccak256, Vec<Vec<u8>>, Vec<u8>>(leaves, leaf_index).encode();

			Ok(proof)
		} else {
			Err(Error {
				code: ErrorCode::InternalError,
				message: format!(
					"Failed to retrieve leaves for commitment_hash {}",
					commitment_hash
				),
				data: None,
			})
		}
	}
}
