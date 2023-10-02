// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};
use frame_support::{assert_ok, traits::EnsureOrigin};
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::{AccountId32, DispatchError::BadOrigin};

#[test]
fn create_agent_bad_origin() {
	new_test_ext().execute_with(|| {
		let alice: AccountId = AccountKeyring::Alice.into();

		let origin = RuntimeOrigin::signed(alice);
		frame_support::assert_noop!(EthereumControl::create_agent(origin), BadOrigin);
	});
}

#[test]
fn create_agent_with_bad_multi_location_yields_location_conversion_failed() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([9; 32]));
		frame_support::assert_noop!(
			EthereumControl::create_agent(origin),
			Error::<Test>::LocationConversionFailed
		);
	});
}

#[test]
fn create_agent_with_bridgehub_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let account = AccountId32::new([1; 32]);
		let origin = RuntimeOrigin::signed(account.clone());
		let _ = Balances::mint_into(&account, 2000);

		let location: MultiLocation =
			<Test as Config>::AgentOrigin::ensure_origin(origin.clone()).unwrap();
		let OriginInfo { agent_id, .. } =
			EthereumControl::process_origin_location(&location).unwrap();

		assert!(!Agents::<Test>::contains_key(agent_id));
		assert_ok!(EthereumControl::create_agent(origin));
		assert!(Agents::<Test>::contains_key(agent_id));
	});
}

#[test]
fn create_agent_with_local_account32_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([2; 32]));
		let expected_agent_id =
			H256(hex!("57fc5659083f0cc883125ccb2c380a1397a3b08434586b8647cc44bcb3647d29"));
		let expected_multi_location = MultiLocation {
			parents: 0,
			interior: X2(Parachain(1013), Junction::AccountId32 { network: None, id: [0; 32] }),
		};

		assert!(!Agents::<Test>::contains_key(expected_agent_id));

		assert_ok!(EthereumControl::create_agent(origin));

		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(expected_multi_location),
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_local_account20_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([3; 32]));
		let expected_agent_id =
			H256(hex!("fc29ec0899cf25874937d04b9b011760fa5dc5cf59af1448abefd389bba7bea2"));
		let expected_multi_location = MultiLocation {
			parents: 0,
			interior: X2(Parachain(1013), AccountKey20 { network: None, key: [0; 20] }),
		};

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(expected_multi_location),
			agent_id: expected_agent_id,
		}));
	});
}

use sp_keyring::AccountKeyring;

#[test]
fn create_agent_with_local_pallet_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([4; 32]));
		let expected_agent_id =
			H256(hex!("ed40c69763094b73c0e3585eeb576fbcee6999123ff1f1beac1f05f5f4c9d945"));
		let expected_multi_location =
			MultiLocation { parents: 0, interior: X2(Parachain(1013), PalletInstance(1)) };

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(expected_multi_location),
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_sibling_chain_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([5; 32]));
		let _ = Balances::mint_into(&AccountId32::new([5; 32]), 2000);

		let expected_agent_id =
			H256(hex!("72456f48efed08af20e5b317abf8648ac66e86bb90a411d9b0b713f7364b75b4"));
		let expected_multi_location = MultiLocation { parents: 0, interior: X1(Parachain(1000)) };

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_ok!(EthereumControl::create_agent(origin));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

	});
}

#[test]
fn create_agent_with_sibling_chain_account32_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([7; 32]));
		let expected_agent_id =
			H256(hex!("fb804b0b77f9c9d69a16d7a45de81225ab8da112e0eb8d2e0229c78086b8927a"));
		let expected_multi_location = MultiLocation {
			parents: 0,
			interior: X2(Parachain(1000), Junction::AccountId32 { network: None, id: [0; 32] }),
		};

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(expected_multi_location),
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_sibling_chain_account20_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([8; 32]));
		let expected_agent_id =
			H256(hex!("74867486f141b159ba1e295bf616d740429269879d4291a12a65eaedbb4b502a"));
		let expected_multi_location = MultiLocation {
			parents: 0,
			interior: X2(Parachain(1000), AccountKey20 { network: None, key: [0; 20] }),
		};

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(expected_multi_location),
			agent_id: expected_agent_id,
		}));
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
fn create_channel_with_sibling_chain_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([5; 32]));

		assert_ok!(EthereumControl::create_agent(origin.clone()));

		assert_ok!(EthereumControl::create_channel(origin));
	});
}

#[test]
fn create_channel_already_exist_yields_failed() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([5; 32]));

		assert_ok!(EthereumControl::create_agent(origin.clone()));

		assert_ok!(EthereumControl::create_channel(origin.clone()));

		frame_support::assert_noop!(
			EthereumControl::create_channel(origin),
			Error::<Test>::ChannelAlreadyCreated
		);
	});
}
