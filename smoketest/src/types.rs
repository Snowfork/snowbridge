use codec::{Decode, Encode};
use scale_info::TypeInfo;
use subxt::ext::sp_core::RuntimeDebug;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum ContractCall {
	V1 {
		/// Target contract address
		target: [u8; 20],
		/// ABI-encoded calldata
		calldata: Vec<u8>,
		/// Include ether held by agent contract
		value: u128,
		/// Maximum gas to forward to target contract
		gas: u64,
	},
}
