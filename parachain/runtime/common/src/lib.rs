#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, parameter_types, PalletId};
use sp_core::H160;
use sp_std::marker::PhantomData;

use snowbridge_core::ChannelId;
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
			ChannelId::Incentivized => {
				incentivized_channel::outbound::Pallet::<T>::submit(who, target, payload)
			},
		}
	}
}

parameter_types! {
	pub const MaxMessagePayloadSize: u32 = 256;
	pub const MaxMessagesPerCommit: u32 = 20;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"s/treasy");
	pub const DotPalletId: PalletId = PalletId(*b"s/dotapp");
}
