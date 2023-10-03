// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};
use frame_support::{assert_ok, assert_noop};
use sp_core::H256;
use sp_runtime::{AccountId32, DispatchError::BadOrigin, TokenError};

#[test]
fn create_agent_bad_origin() {
	new_test_ext().execute_with(|| {
		frame_support::assert_noop!(EthereumControl::create_agent(RuntimeOrigin::signed([0; 32].into())), BadOrigin);
		frame_support::assert_noop!(EthereumControl::create_agent(RuntimeOrigin::none()), BadOrigin);
	});
}

#[test]
fn create_agent_success() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };
		let agent_id = agent_id_of(&origin_location).unwrap();
		let sovereign_account = sovereign_account_of(&origin_location).unwrap();

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 2000);

		assert!(!Agents::<Test>::contains_key(agent_id));

		let origin = make_xcm_origin(origin_location);
		assert_ok!(EthereumControl::create_agent(origin));

		assert!(Agents::<Test>::contains_key(agent_id));
	});
}

#[test]
fn create_agent_for_sibling_fail_not_enough_funds() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };

		let origin = make_xcm_origin(origin_location);
		assert_noop!(EthereumControl::create_agent(origin), TokenError::FundsUnavailable);
	});
}

#[test]
fn upgrade_without_root_yields_bad_origin() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([0; 32]));
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();

		frame_support::assert_noop!(
			EthereumControl::upgrade(origin, address, code_hash, None),
			BadOrigin
		);
	});
}

#[test]
fn upgrade_with_root_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();

		frame_support::assert_ok!(EthereumControl::upgrade(origin, address, code_hash, None));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::Upgrade {
			impl_address: address,
			impl_code_hash: code_hash,
			params_hash: None,
		}));
	});
}

#[test]
fn upgrade_with_params_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let initializer: Option<Initializer> = Some(Initializer{params: [0; 256].into(), maximum_required_gas: 10000});
		frame_support::assert_ok!(EthereumControl::upgrade(origin, address, code_hash, initializer));
	});
}

#[test]
fn create_channel_success() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };
		let sovereign_account = sovereign_account_of(&origin_location).unwrap();
		let origin = make_xcm_origin(origin_location);

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 10000);

		assert_ok!(EthereumControl::create_agent(origin.clone()));
		assert_ok!(EthereumControl::create_channel(origin));
	});
}

#[test]
fn create_channel_already_exist_yields_failed() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };
		let sovereign_account = sovereign_account_of(&origin_location).unwrap();
		let origin = make_xcm_origin(origin_location);

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 10000);

		assert_ok!(EthereumControl::create_agent(origin.clone()));
		assert_ok!(EthereumControl::create_channel(origin.clone()));

		frame_support::assert_noop!(
			EthereumControl::create_channel(origin),
			Error::<Test>::ChannelAlreadyCreated
		);
	});
}
