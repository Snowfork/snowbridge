// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::mock::*;
use crate::*;
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::AccountId32;
use sp_runtime::DispatchError::BadOrigin;

#[test]
fn create_agent_with_unknown_origin_yields_bad_origin() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([0; 32]));
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
fn create_agent_with_failed_validate_yields_submission_failed() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([10; 32]));
		frame_support::assert_noop!(
			EthereumControl::create_agent(origin),
			Error::<Test>::SubmissionFailed
		);
	});
}

#[test]
fn create_agent_with_failed_submit_yields_submission_failed() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([11; 32]));
		frame_support::assert_noop!(
			EthereumControl::create_agent(origin),
			Error::<Test>::SubmissionFailed
		);
	});
}

#[test]
fn create_agent_is_idempotent() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([1; 32]));
		let expected_agent_id =
			H256(hex!("d9380024e49afa1ac89c0127fea210bb6b431b10dafefab8061bd88ac25d17a5"));

		Agents::<Test>::insert(expected_agent_id, ());

		assert!(Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		assert_eq!(System::events().len(), 0);
	});
}

#[test]
fn create_agent_with_relaychain_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([1; 32]));
		let expected_agent_id =
			H256(hex!("d9380024e49afa1ac89c0127fea210bb6b431b10dafefab8061bd88ac25d17a5"));
		let expected_multi_location =
			VersionedMultiLocation::V3(MultiLocation { parents: 1, interior: Here });

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_local_account32_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([2; 32]));
		let expected_agent_id =
			H256(hex!("9e85ef53611dcb973a337977a79217890f6c0d605de20ae4a828b1b9a95162c4"));
		let expected_multi_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: X1(Junction::AccountId32 { network: None, id: [0; 32] }),
		});

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_local_account20_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([3; 32]));
		let expected_agent_id =
			H256(hex!("927a4def0d0bdd151dfa247a07e4036e12335ee71977426847be6e6e36e3c460"));
		let expected_multi_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: X1(AccountKey20 { network: None, key: [0; 20] }),
		});

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_local_pallet_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([4; 32]));
		let expected_agent_id =
			H256(hex!("964c3b3f978db1febb282d675dcf2196eae3c28fd7c0885b738cee828262fcc2"));
		let expected_multi_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 0,
			interior: X1(PalletInstance(1)),
		});

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_sibling_chain_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([5; 32]));
		let expected_agent_id =
			H256(hex!("81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79"));
		let expected_multi_location =
			VersionedMultiLocation::V3(MultiLocation { parents: 1, interior: X1(Parachain(1000)) });

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_sibling_chain_account32_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([7; 32]));
		let expected_agent_id =
			H256(hex!("75ad585e231db5daf900819e8fb62af432610619d0d7a1156e5d78531b2c6493"));
		let expected_multi_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 1,
			interior: X2(Parachain(1000), Junction::AccountId32 { network: None, id: [0; 32] }),
		});

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}

#[test]
fn create_agent_with_sibling_chain_account20_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([8; 32]));
		let expected_agent_id =
			H256(hex!("3a737a558137d674c5e9c49bd0e6389bf69e1825c8fd531af5534081016501ef"));
		let expected_multi_location = VersionedMultiLocation::V3(MultiLocation {
			parents: 1,
			interior: X2(Parachain(1000), AccountKey20 { network: None, key: [0; 20] }),
		});

		assert!(!Agents::<Test>::contains_key(expected_agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(expected_agent_id));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			agent_location: expected_multi_location,
			agent_id: expected_agent_id,
		}));
	});
}
