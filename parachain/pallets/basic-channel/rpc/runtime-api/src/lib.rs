#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait BasicOutboundChannelApi {
		fn generate_proof(leaf_index: u64) ->  Result<u64, ()>;
	}
}
