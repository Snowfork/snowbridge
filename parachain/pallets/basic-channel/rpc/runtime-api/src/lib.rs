#![cfg_attr(not(feature = "std"), no_std)]

// #[derive(RuntimeDebug, codec::Encode, codec::Decode, PartialEq, Eq)]
// pub enum GenerateProofApiError {
// 	/// Error getting the new root.
// 	GetRoot,
// 	/// Error during proof generation.
// 	GenerateProof,
// 	/// Leaf not found in the storage.
// 	LeafNotFound,
// }

sp_api::decl_runtime_apis! {
	pub trait BasicOutboundChannelApi {
		fn generate_proof(leaves: Vec<Vec<u8>>, leaf_index: u64) ->  Result<Vec<u8>, ()>;
	}
}
