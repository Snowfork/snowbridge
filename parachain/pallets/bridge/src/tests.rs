use frame_support::storage::StorageMap;
use frame_support::{assert_noop, assert_ok};
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use artemis_core::{ChannelId, Message, Proof};

use hex_literal::hex;

use crate::{
	channel::outbound::make_outbound_channel,
	mock::{new_tester, new_tester_with_source_channels, AccountId, Bridge, Origin, Test},
	primitives::{InboundChannelData, OutboundChannelData},
	Error, InboundChannels, OutboundChannels,
};

#[test]
#[ignore] // TODO: what happens with the bridge
fn test_submit_outbound_basic() {
	new_tester().execute_with(|| {
		let chan_id = ChannelId::Basic;
		let target = H160::zero();
		let channel = make_outbound_channel::<Test>(chan_id);

		assert_ok!(channel.submit(target, &vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		assert_ok!(channel.submit(target, &vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}

#[test]
#[ignore] // TODO: what happens with the bridge
fn test_submit_outbound_incentivized() {
	new_tester().execute_with(|| {
		let chan_id = ChannelId::Incentivized;
		let target = H160::zero();
		let channel = make_outbound_channel::<Test>(chan_id);

		assert_ok!(channel.submit(target, &vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		assert_ok!(channel.submit(target, &vec![0, 1, 2]));

		let data: OutboundChannelData = OutboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}

// The originating channel address for the messages below
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["2d02f2234d0B6e35D8d8fD77705f535ACe681327"];

// Ethereum Log:
//   address: 0xe4ab635d0bdc5668b3fcb4eaee1dec587998f4af (outbound channel contract)
//   topics: ...
//   data:
//     source: 0x8f5acf5f15d4c3d654a759b96bb674a236c8c0f3  (ETH bank contract)
//     nonce: 1
//     payload ...
const MESSAGE_DATA_0: [u8; 284] = hex!(
	"
	f90119942d02f2234d0b6e35d8d8fd77705f535ace681327e1a0779b38144a38
	cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15bb8e000000000
	00000000000000000a42cba2b7960a0ce216ade5d6a82574257023d800000000
	0000000000000000000000000000000000000000000000000000000100000000
	0000000000000000000000000000000000000000000000000000006000000000
	000000000000000000000000000000000000000000000000000000570c018213
	dae5f9c236beab905c8305cb159c5fa1aae500d43593c715fdd31c61141abd04
	a99fd6822c8558854ccde39a5684e7a56da27d0000d9e9ac2d78030000000000
	00000000000000000000000000000000000000000000000000000000
"
);

// Ethereum Log:
//   address: 0xe4ab635d0bdc5668b3fcb4eaee1dec587998f4af (outbound channel contract)
//   topics: ...
//   data:
//     source: 0x8f5acf5f15d4c3d654a759b96bb674a236c8c0f3  (ETH bank contract)
//     nonce: 1
//     payload ...
const MESSAGE_DATA_1: [u8; 284] = hex!(
	"
	f90119942d02f2234d0b6e35d8d8fd77705f535ace681327e1a0779b38144a38
	cfc4351816442048b17fe24ba2b0e0c63446b576e8281160b15bb8e000000000
	00000000000000000a42cba2b7960a0ce216ade5d6a82574257023d800000000
	0000000000000000000000000000000000000000000000000000000200000000
	0000000000000000000000000000000000000000000000000000006000000000
	000000000000000000000000000000000000000000000000000000570c018213
	dae5f9c236beab905c8305cb159c5fa1aae500d43593c715fdd31c61141abd04
	a99fd6822c8558854ccde39a5684e7a56da27d0000d9e9ac2d78030000000000
	00000000000000000000000000000000000000000000000000000000
"
);

#[test]
fn test_submit_inbound_invalid_source_channel() {
	new_tester_with_source_channels(H160::zero(), H160::zero()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_noop!(
			Bridge::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidSourceChannel
		);
	});
}

#[test]
fn test_submit_inbound_basic() {
	new_tester_with_source_channels(SOURCE_CHANNEL_ADDR.into(), H160::zero()).execute_with(|| {
		let chan_id = ChannelId::Basic;
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message_1));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		// Submit message 2
		let message_2 = Message {
			data: MESSAGE_DATA_1.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message_2));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}
#[test]
fn test_submit_inbound_basic_bad_nonce() {
	new_tester_with_source_channels(SOURCE_CHANNEL_ADDR.into(), H160::zero()).execute_with(|| {
		let chan_id = ChannelId::Basic;
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message.clone()));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		// Submit the same again
		assert_noop!(
			Bridge::submit(origin.clone(), message.clone()),
			Error::<Test>::BadNonce
		);
	});
}

#[test]
fn test_submit_inbound_incentivized() {
	new_tester_with_source_channels(H160::zero(), SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let chan_id = ChannelId::Incentivized;
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message_1));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		// Submit message 2
		let message_2 = Message {
			data: MESSAGE_DATA_1.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message_2));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 2);
	});
}
#[test]
fn test_submit_inbound_incentivized_bad_nonce() {
	new_tester_with_source_channels(H160::zero(), SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let chan_id = ChannelId::Incentivized;
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);

		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(Bridge::submit(origin.clone(), message.clone()));
		let data: InboundChannelData = InboundChannels::get(chan_id);
		assert_eq!(data.nonce, 1);

		// Submit the same again
		assert_noop!(
			Bridge::submit(origin.clone(), message.clone()),
			Error::<Test>::BadNonce
		);
	});
}
