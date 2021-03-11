use crate::Config;
use crate::mock::{Test, AccountId, Balances, DOTApp, Event, Origin, System, new_tester};
use frame_support::{assert_noop, assert_ok,
	dispatch::{
		DispatchError,
	},
	traits::Currency,
};
use sp_keyring::AccountKeyring as Keyring;
use sp_core::H160;
use artemis_core::ChannelId;

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}


#[test]
fn should_lock() {
	new_tester().execute_with(|| {
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(2);
		let amount = 100;

		let _ = Balances::deposit_creating(&sender, amount * 2);

		assert_ok!(DOTApp::lock(
			Origin::signed(sender.clone()),
			ChannelId::Incentivized,
			recipient.clone(),
			amount));

		assert_eq!(Balances::total_balance(&DOTApp::account_id()), amount);

		assert_eq!(
			Event::dot_app(crate::Event::<Test>::Locked(sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn should_unlock() {
	new_tester().execute_with(|| {
		let peer_contract = H160::repeat_byte(1);
		let sender = H160::repeat_byte(7);
		let recipient: AccountId = Keyring::Bob.into();
		let balance = 500;
		let amount = 100;
		let amount_wrapped = crate::primitives::wrap::<Test>(amount, <Test as Config>::Decimals::get()).unwrap();

		let _ = Balances::deposit_creating(&DOTApp::account_id(), balance);

		assert_ok!(
			DOTApp::unlock(
				artemis_dispatch::Origin(peer_contract).into(),
				sender,
				recipient.clone(),
				amount_wrapped,
			)
		);
		assert_eq!(Balances::total_balance(&recipient), amount);
		assert_eq!(Balances::total_balance(&DOTApp::account_id()), balance - amount);

		assert_eq!(
			Event::dot_app(crate::Event::<Test>::Unlocked(sender, recipient, amount)),
			last_event()
		);
	});
}

#[test]
fn should_not_unlock_on_bad_origin_failure() {
	new_tester().execute_with(|| {
		let unknown_peer_contract = H160::repeat_byte(64);
		let sender = H160::repeat_byte(7);
		let recipient: AccountId = Keyring::Bob.into();
		let balance = 500;
		let amount = 100;
		let amount_wrapped = crate::primitives::wrap::<Test>(amount, <Test as Config>::Decimals::get()).unwrap();

		let _ = Balances::deposit_creating(&DOTApp::account_id(), balance);

		assert_noop!(
			DOTApp::unlock(
				artemis_dispatch::Origin(unknown_peer_contract).into(),
				sender,
				recipient.clone(),
				amount_wrapped,
			),
			DispatchError::BadOrigin
		);

		assert_noop!(
			DOTApp::unlock(
				Origin::signed(Keyring::Alice.into()),
				sender,
				recipient.clone(),
				amount_wrapped,
			),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn should_not_lock_on_add_commitment_failure() {
	new_tester().execute_with(|| {
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(9);
		let amount = 100;

		let _ = Balances::deposit_creating(&sender, amount * 10);

		assert_noop!(
			DOTApp::lock(
				Origin::signed(sender.clone()),
				ChannelId::Basic,
				recipient.clone(),
				amount.into()
			),
			DispatchError::Other("some error!")
		);
	});
}
