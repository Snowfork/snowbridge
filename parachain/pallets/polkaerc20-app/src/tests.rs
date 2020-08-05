use crate::{mock::*};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use frame_support::storage::StorageDoubleMap;
use hex::FromHex;

use artemis_ethereum::Event;
use crate::FreeBalance;

fn to_account_id(hexaddr: &str) -> [u8; 32] {
	let mut buf: [u8; 32] = [0; 32];
	let bytes: Vec<u8> = hexaddr.from_hex().unwrap();
	buf.clone_from_slice(&bytes);
	buf
}

#[test]
fn it_mints() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();
		let alice: AccountId = Keyring::Alice.into();
		assert_ok!(PolkaERC20::do_mint(token_addr, &alice, 500));
		assert_eq!(FreeBalance::<MockRuntime>::get(&token_addr, &alice), 500);

		assert_ok!(PolkaERC20::do_mint(token_addr, &alice, 20));
		assert_eq!(FreeBalance::<MockRuntime>::get(&token_addr, &alice), 520);
	});
}

#[test]
fn it_handles_ethereum_event() {
	new_tester().execute_with(|| {
		let token_addr = H160::zero();

		let event = Event::SendERC20 {
			sender: "cffeaaf7681c89285d65cfbe808b80e502696573".parse().unwrap(),
			recipient: to_account_id("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
			token: token_addr,
			amount: 10.into(),
			nonce: 1
		};

		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(PolkaERC20::handle_event(event));
		assert_eq!(FreeBalance::<MockRuntime>::get(&token_addr, &bob), 10);
	});
}