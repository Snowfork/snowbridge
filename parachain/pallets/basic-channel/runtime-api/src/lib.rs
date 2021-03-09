//! Runtime API definition for the Basic Channel API

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	pub trait BasicChannelApi<AccountId> where
		AccountId: codec::Codec,
	{
		fn get_merkle_proofs(root: sp_core::H256) -> u64;
	}
}
