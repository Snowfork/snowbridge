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

use snowbridge_core::{AssetId, ChannelId};

// This function replicates BlockWeights::with_sensible_defaults but uses custom
// base block and extrinsic weights.
// https://github.com/paritytech/substrate/blob/4898eb350d636d439827cb43f0f26e72e66492fa/frame/system/src/limits.rs#L312
pub fn build_block_weights(
	base_block_weight: Weight,
	base_extrinsic_weight: Weight,
	expected_block_weight: Weight,
	normal_ratio: Perbill,
) -> BlockWeights {
	let normal_weight = normal_ratio * expected_block_weight;

	BlockWeights::builder()
		.base_block(base_block_weight)
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = base_extrinsic_weight.into();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = normal_weight.into();
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = expected_block_weight.into();
			weights.reserved = (expected_block_weight - normal_weight).into();
		})
		.avg_block_initialization(Perbill::from_percent(10))
		.build()
		.expect("Weights must be valid")
}

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
	pub const Ether: AssetId = AssetId::ETH;
	pub const MaxMessagePayloadSize: u64 = 256;
	pub const MaxMessagesPerCommit: u64 = 20;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"s/treasy");
	pub const DotPalletId: PalletId = PalletId(*b"s/dotapp");
}
