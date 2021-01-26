use frame_support::assert_ok;
use frame_support::storage::StorageMap;

use artemis_core::ChannelId;

use crate::{
	mock::{new_tester, Test},
	OutboundChannels,
	channel::outbound::make_outbound_channel,
	primitives::OutboundChannelData
};

#[test]
fn test_submit_outbound_basic() {
	new_tester().execute_with(|| {
		let chan_id = ChannelId::Basic;
		let channel = make_outbound_channel::<Test>(chan_id);

		assert_ok!(channel.submit(&vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		assert_ok!(channel.submit(&vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}

#[test]
fn test_submit_outbound_incentivized() {
	new_tester().execute_with(|| {
		let chan_id = ChannelId::Incentivized;
		let channel = make_outbound_channel::<Test>(chan_id);

		assert_ok!(channel.submit(&vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		assert_ok!(channel.submit(&vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}
