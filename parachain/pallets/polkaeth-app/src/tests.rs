use crate::{mock::*};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use hex::FromHex;

use artemis_ethereum::Event;

fn to_account_id(hexaddr: &str) -> [u8; 32] {
	let mut buf: [u8; 32] = [0; 32];
	let bytes: Vec<u8> = hexaddr.from_hex().unwrap();
	buf.clone_from_slice(&bytes);
	buf
}

#[test]
fn it_mints() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(Keyring::Alice.into());
		PolkaETHModule::do_mint(Keyring::Bob.into(), 42);

		let bob: AccountId = Keyring::Bob.into();

		assert_eq!(BalancesPolkaETH::free_balance(bob), 1042);
	});
}

#[test]
fn it_burns() {
	new_tester().execute_with(|| {
		let origin = Origin::signed(Keyring::Alice.into());
		assert_ok!(PolkaETHModule::burn(origin, 500, true));

		let alice: AccountId = Keyring::Alice.into();

		assert_eq!(BalancesPolkaETH::free_balance(alice), 500);
	});
}


#[test]
fn it_handles_ethereum_event() {
	new_tester().execute_with(|| {
		let bob: AccountId = Keyring::Bob.into();

		assert_eq!(BalancesPolkaETH::free_balance(bob.clone()), 1000);

		let token_addr = H160::zero();

		let event = Event::SendETH {
			sender: "cffeaaf7681c89285d65cfbe808b80e502696573".parse().unwrap(),
			recipient: to_account_id("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
			amount: 1000.into(),
			nonce: 1
		};


		assert_ok!(PolkaETHModule::handle_event(event));
		assert_eq!(BalancesPolkaETH::free_balance(bob.clone()), 2000);
	});
}
