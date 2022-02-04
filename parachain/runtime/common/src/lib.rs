#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	weights::{DispatchClass, Weight},
	PalletId,
};
use frame_system::limits::BlockWeights;
use sp_core::H160;
use sp_runtime::Perbill;
use sp_std::marker::PhantomData;

use snowbridge_core::ChannelId;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

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
pub type DigestItem = generic::DigestItem<Hash>;

/// Aura consensus authority.
pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;

pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Hash algorithm.
	pub type Hasher = sp_runtime::traits::BlakeTwo256;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

pub const MILLISECS_PER_BLOCK: u64 = 12000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub const INDEXING_PREFIX: &'static [u8] = b"commitment";
pub struct OutboundRouter<T>(PhantomData<T>);

impl<T> snowbridge_core::OutboundRouter<T::AccountId> for OutboundRouter<T>
where
	T: basic_channel::outbound::Config + incentivized_channel::outbound::Config,
{
	fn submit(
		channel_id: ChannelId,
		who: &T::AccountId,
		target: H160,
		payload: &[u8],
	) -> DispatchResult {
		match channel_id {
			ChannelId::Basic => basic_channel::outbound::Pallet::<T>::submit(who, target, payload),
			ChannelId::Incentivized =>
				incentivized_channel::outbound::Pallet::<T>::submit(who, target, payload),
		}
	}
}

parameter_types! {
	pub const MaxMessagePayloadSize: u64 = 256;
	pub const MaxMessagesPerCommit: u64 = 20;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"s/treasy");
	pub const DotPalletId: PalletId = PalletId(*b"s/dotapp");
}
