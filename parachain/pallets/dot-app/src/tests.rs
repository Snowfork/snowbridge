use crate::{
	mock::{new_tester, AccountId, Balances, DotApp, RuntimeEvent, RuntimeOrigin, System, Test},
	Config,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, traits::Currency};
use snowbridge_core::ChannelId;
use sp_core::H160;
use sp_keyring::AccountKeyring as Keyring;

fn last_event() -> RuntimeEvent {
	System::events().pop().expect("Event expected").event
}

#[test]
fn should_lock() {
	new_tester().execute_with(|| {
		let sender: AccountId = Keyring::Bob.into();
		let recipient = H160::repeat_byte(2);
		let amount = 100;

		let _ = Balances::deposit_creating(&sender, amount * 2);

		assert_ok!(DotApp::lock(
			RuntimeOrigin::signed(sender.clone()),
			ChannelId::Incentivized,
			recipient.clone(),
			amount
		));

		assert_eq!(Balances::total_balance(&DotApp::account_id().unwrap()), amount);

		assert_eq!(
			RuntimeEvent::DotApp(crate::Event::<Test>::Locked(sender, recipient, amount)),
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
		let amount_wrapped =
			crate::primitives::wrap::<Test>(amount, <Test as Config>::Decimals::get()).unwrap();

		let _ = Balances::deposit_creating(&DotApp::account_id().unwrap(), balance);

		assert_ok!(DotApp::unlock(
			snowbridge_dispatch::RawOrigin(peer_contract).into(),
			sender,
			recipient.clone(),
			amount_wrapped,
		));
		assert_eq!(Balances::total_balance(&recipient), amount);
		assert_eq!(Balances::total_balance(&DotApp::account_id().unwrap()), balance - amount);

		assert_eq!(
			RuntimeEvent::DotApp(crate::Event::<Test>::Unlocked(sender, recipient, amount)),
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
		let amount_wrapped =
			crate::primitives::wrap::<Test>(amount, <Test as Config>::Decimals::get()).unwrap();

		let _ = Balances::deposit_creating(&DotApp::account_id().unwrap(), balance);

		assert_noop!(
			DotApp::unlock(
				snowbridge_dispatch::RawOrigin(unknown_peer_contract).into(),
				sender,
				recipient.clone(),
				amount_wrapped,
			),
			DispatchError::BadOrigin
		);

		assert_noop!(
			DotApp::unlock(
				RuntimeOrigin::signed(Keyring::Alice.into()),
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

		for _ in 0..3 {
			let _ = DotApp::lock(
				RuntimeOrigin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				1,
			);
		}

		assert_noop!(
			DotApp::lock(
				RuntimeOrigin::signed(sender.clone()),
				ChannelId::Incentivized,
				recipient.clone(),
				amount
			),
			snowbridge_incentivized_channel::outbound::Error::<Test>::QueueSizeLimitReached
		);
	});
}
