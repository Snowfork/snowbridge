// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};
use frame_support::{assert_ok, traits::EnsureOrigin};
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::{AccountId32, DispatchError::BadOrigin};
use sp_core::H160;
use xcm::prelude::AccountKey20;
use xcm::v3::{Junction, MultiLocation};
use xcm::v3::Junction::Parachain;
use xcm::v3::Junctions::X2;

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
			Error::<Test>::LocationToAgentIdConversionFailed
		);
	});
}

#[test]
fn create_agent_with_bridgehub_origin_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([1; 32]));

		let location: MultiLocation =
			<Test as Config>::AgentOrigin::ensure_origin(origin.clone()).unwrap();
		let (agent_id, _, location) = EthereumControl::convert_location(location).unwrap();

		assert!(!Agents::<Test>::contains_key(agent_id));
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
		assert!(Agents::<Test>::contains_key(agent_id));

		// println!("agent_id: {:#?}", hex::encode(agent_id.as_bytes()));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::CreateAgent {
			location: Box::new(location),
			agent_id,
		}));
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
		assert_eq!(EthereumControl::create_agent(origin), Ok(()));
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
		let expected_agent_id =
			H256(hex!("72456f48efed08af20e5b317abf8648ac66e86bb90a411d9b0b713f7364b75b4"));
		let expected_multi_location = MultiLocation { parents: 0, interior: X1(Parachain(1000)) };

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
fn create_agent_without_root_yields_bad_origin() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([0; 32]));
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let params: Option<Vec<u8>> = None;

		frame_support::assert_noop!(
			EthereumControl::upgrade(origin, address, code_hash, params),
			BadOrigin
		);
	});
}

#[test]
fn create_agent_with_root_yields_success() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let params: Option<Vec<u8>> = None;
		let expected_hash = None;

		frame_support::assert_ok!(EthereumControl::upgrade(origin, address, code_hash, params));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::Upgrade {
			impl_address: address,
			impl_code_hash: code_hash,
			params_hash: expected_hash,
		}));
	});
}

#[test]
fn create_agent_with_large_params_yields_upgrade_too_large() {
	new_test_ext().execute_with(|| {
		const MAX_SIZE: usize = MaxUpgradeDataSize::get() as usize;
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let params: Option<Vec<u8>> = Some([0; MAX_SIZE].into());

		frame_support::assert_noop!(
			EthereumControl::upgrade(origin, address, code_hash, params),
			Error::<Test>::UpgradeDataTooLarge
		);
	});
}

#[test]
fn create_agent_with_small_params_yields_success() {
	new_test_ext().execute_with(|| {
		const MAX_SIZE_LESS_ONE: usize = (MaxUpgradeDataSize::get() - 1) as usize;
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let params: Option<Vec<u8>> = Some([0; MAX_SIZE_LESS_ONE].into());
		let expected_hash =
			Some(H256(hex!("c95ef6b0bf891c06e1318f07b86977998674a0ae996999915c1f5d93359e72a9")));

		frame_support::assert_ok!(EthereumControl::upgrade(origin, address, code_hash, params));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::Upgrade {
			impl_address: address,
			impl_code_hash: code_hash,
			params_hash: expected_hash,
		}));
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
fn create_channel_with_sibling_chain_pallet_as_origin_yields_location_conversion_failed() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([6; 32]));

		assert_ok!(EthereumControl::create_agent(origin.clone()));

		frame_support::assert_noop!(
			EthereumControl::create_channel(origin),
			Error::<Test>::LocationToParaIdConversionFailed
		);
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
