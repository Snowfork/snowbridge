use crate::mock::{new_tester, AccountId, Balances, DOTApp, Event, Origin, System, Test};
use artemis_core::ChannelId;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, traits::Currency};
use sp_core::{H160, U256};
use sp_keyring::AccountKeyring as Keyring;

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
			amount
		));

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
		let amount = 100;
		let balance = 500;

		let _ = Balances::deposit_creating(&DOTApp::account_id(), balance);

		assert_ok!(DOTApp::unlock(
			artemis_dispatch::Origin(peer_contract).into(),
			sender,
			recipient.clone(),
			amount
		));
		assert_eq!(Balances::total_balance(&recipient), amount);
		assert_eq!(
			Balances::total_balance(&DOTApp::account_id()),
			balance - amount
		);

		assert_eq!(
			Event::dot_app(crate::Event::<Test>::Unlocked(
				sender,
				recipient,
				amount.into()
			)),
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
		let amount = 100;
		let balance = 500;

		let _ = Balances::deposit_creating(&DOTApp::account_id(), balance);

		assert_noop!(
			DOTApp::unlock(
				artemis_dispatch::Origin(unknown_peer_contract).into(),
				sender,
				recipient.clone(),
				amount
			),
			DispatchError::BadOrigin
		);

		assert_noop!(
			DOTApp::unlock(
				Origin::signed(Keyring::Alice.into()),
				sender,
				recipient.clone(),
				amount
			),
			DispatchError::BadOrigin
		);

		assert_eq!(Balances::total_balance(&DOTApp::account_id()), balance);
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
				amount
			),
			DispatchError::Other("some error!")
		);
	});
}

// Used to prove safety of conversion from DOT to wrapped DOT (See BaseDOTApp.sol)
#[test]
fn should_max_dot_convert_to_wrapped_dot() {
	let granularity = U256::from(100000000u64); // 10 ** 8
	U256::from(u128::MAX).checked_mul(granularity).unwrap();
}
