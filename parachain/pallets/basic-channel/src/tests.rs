use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

use artemis_core::Proof;

use super::*;
use crate::mock::*;

#[test]
fn test_submit_outbound_basic() {
	new_tester().execute_with(|| {
		let account = &AccountId::default();
		let target = H160::zero();

		let nonce = OutboundChannels::<Test>::get(account);
		assert_eq!(nonce, 0);

		assert_ok!(Module::<Test>::submit_outbound(
			account,
			target,
			&vec![0, 1, 2]
		));

		let nonce = OutboundChannels::<Test>::get(account);
		assert_eq!(nonce, 1);
	});
}

// The originating channel address for the messages below
const SOURCE_CHANNEL_ADDR: [u8; 20] = hex!["2ffa5ecdbe006d30397c7636d3e015eee251369f"];

// Ethereum Log:
//   origin: 0x89b4ab1ef20763630df9743acf155865600daff2 (origin address)
//   source: 0x774667629726ec1fabebcec0d9139bd1c8f72a23 (outbound channel contract)
const MESSAGE_DATA_0: [u8; 317] = hex!(
	"
        f9013a942ffa5ecdbe006d30397c7636d3e015eee251369fe1a0daab80e89869
        997d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb90100000000
        00000000000000000089b4ab1ef20763630df9743acf155865600daff2000000
        000000000000000000774667629726ec1fabebcec0d9139bd1c8f72a23000000
        0000000000000000000000000000000000000000000000000000000001000000
        0000000000000000000000000000000000000000000000000000000080000000
        00000000000000000000000000000000000000000000000000000000570c0189
        b4ab1ef20763630df9743acf155865600daff200d43593c715fdd31c61141abd
        04a99fd6822c8558854ccde39a5684e7a56da27d0000c16ff286230000000000
        0000000000000000000000000000000000000000000000000000000000
"
);

// Ethereum Log:
// TODO
const MESSAGE_DATA_1: [u8; 317] = hex!(
	"
        f9013a942ffa5ecdbe006d30397c7636d3e015eee251369fe1a0daab80e89869
        997d1cabbe1122788e90fe72b9234ff97a9217dcbb5126f3562fb90100000000
        00000000000000000089b4ab1ef20763630df9743acf155865600daff2000000
        000000000000000000774667629726ec1fabebcec0d9139bd1c8f72a23000000
        0000000000000000000000000000000000000000000000000000000001000000
        0000000000000000000000000000000000000000000000000000000080000000
        00000000000000000000000000000000000000000000000000000000570c0189
        b4ab1ef20763630df9743acf155865600daff200d43593c715fdd31c61141abd
        04a99fd6822c8558854ccde39a5684e7a56da27d0000c16ff286230000000000
        0000000000000000000000000000000000000000000000000000000000
"
);

#[test]
fn test_submit_inbound_invalid_source_channel() {
	new_tester_with_source_channel(H160::zero()).execute_with(|| {
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
			BasicChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::InvalidSourceChannel
		);
	});
}

#[test]
fn test_submit_inbound_basic() {
	new_tester_with_source_channel(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);
		let eth_address = H160::from_slice(&hex!("0a42cba2b7960a0ce216ade5d6a82574257023d8")[..]);

		// Submit message 1
		let message_1 = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};

		assert_ok!(BasicChannel::submit(origin.clone(), message_1));
		let nonce = InboundChannels::get(eth_address);
		assert_eq!(nonce, 1);

		// Submit message 2
		let message_2 = Message {
			data: MESSAGE_DATA_1.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(BasicChannel::submit(origin.clone(), message_2));
		let nonce = InboundChannels::get(eth_address);
		assert_eq!(nonce, 2);
	});
}

#[test]
fn test_submit_inbound_basic_bad_nonce() {
	new_tester_with_source_channel(SOURCE_CHANNEL_ADDR.into()).execute_with(|| {
		let relayer: AccountId = Keyring::Bob.into();
		let origin = Origin::signed(relayer);
		let tx_origin = H160::from_slice(&hex!("89b4ab1ef20763630df9743acf155865600daff2")[..]);
		// Submit message
		let message = Message {
			data: MESSAGE_DATA_0.into(),
			proof: Proof {
				block_hash: Default::default(),
				tx_index: Default::default(),
				data: Default::default(),
			},
		};
		assert_ok!(BasicChannel::submit(origin.clone(), message.clone()));
		let nonce = InboundChannels::get(tx_origin);
		assert_eq!(nonce, 1);

		// Submit the same again
		assert_noop!(
			BasicChannel::submit(origin.clone(), message.clone()),
			Error::<Test>::BadNonce
		);
	});
}
