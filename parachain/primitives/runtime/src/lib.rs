#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{
	generic, MultiSignature, MultiAddress,
	traits::{Verify, BlakeTwo256, IdentifyAccount},
};

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// Opaque header
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Opaque block
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Opaque block id
pub type BlockId = generic::BlockId<Block>;

