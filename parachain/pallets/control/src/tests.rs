// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_runtime::{AccountId32, DispatchError::BadOrigin, TokenError, traits::AccountIdConversion};

#[test]
fn create_agent_for_sibling() {
	new_test_ext().execute_with(|| {
		let origin_para_id = 2000;
		let origin_location = MultiLocation {
			parents: 1,
			interior: X1(Parachain(origin_para_id)),
		};
		let agent_id = make_agent_id(origin_location);
		let sovereign_account = ParaId::from(origin_para_id).into_account_truncating();

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 10000);

		assert!(!Agents::<Test>::contains_key(agent_id));

		let origin = make_xcm_origin(origin_location);
		assert_ok!(EthereumControl::create_agent(origin));

		assert!(Agents::<Test>::contains_key(agent_id));
	});
}

#[test]
fn create_agent_for_sibling_fails_on_funds_unavailable() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };

		let origin = make_xcm_origin(origin_location);
		assert_noop!(EthereumControl::create_agent(origin), TokenError::FundsUnavailable);
	});
}

#[test]
fn create_agent_for_sibling_child() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation {
			parents: 1,
			interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
		};
		let agent_id = make_agent_id(origin_location);

		// Create channel for sibling parachain
		Channels::<Test>::insert(ParaId::from(2000), ());

		assert!(!Agents::<Test>::contains_key(agent_id));

		let origin = make_xcm_origin(origin_location);
		assert_ok!(EthereumControl::create_agent(origin));

		assert!(Agents::<Test>::contains_key(agent_id));
	});
}

#[test]
fn create_agent_for_sibling_child_fails_no_channel() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation {
			parents: 1,
			interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
		};

		let origin = make_xcm_origin(origin_location);
		assert_noop!(
			EthereumControl::create_agent(origin),
			Error::<Test>::ChannelNotExist
		);
	});
}

#[test]
fn create_agent_bad_origin() {
	new_test_ext().execute_with(|| {
		// relay chain location not allowed
		assert_noop!(
			EthereumControl::create_agent(
				make_xcm_origin(
					MultiLocation {
						parents: 1,
						interior: Here,
					}
				)
			),
			BadOrigin,
		);

		// local account location not allowed
		assert_noop!(
			EthereumControl::create_agent(
				make_xcm_origin(
					MultiLocation {
						parents: 0,
						interior: X1(Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}
				)
			),
			BadOrigin,
		);

		// Signed origin not allowed
		assert_noop!(
			EthereumControl::create_agent(RuntimeOrigin::signed([14; 32].into())),
			BadOrigin
		);

		// None origin not allowed
		assert_noop!(
			EthereumControl::create_agent(RuntimeOrigin::none()),
			BadOrigin
		);
	});
}

#[test]
fn upgrade_as_root() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();

		assert_ok!(EthereumControl::upgrade(origin, address, code_hash, None));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::Upgrade {
			impl_address: address,
			impl_code_hash: code_hash,
			initializer_params_hash: None,
		}));
	});
}

#[test]
fn upgrade_as_signed_fails() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::new([0; 32]));
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();

		assert_noop!(
			EthereumControl::upgrade(origin, address, code_hash, None),
			BadOrigin
		);
	});
}

#[test]
fn upgrade_with_params() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let address: H160 = Default::default();
		let code_hash: H256 = Default::default();
		let initializer: Option<Initializer> =
			Some(Initializer { params: [0; 256].into(), maximum_required_gas: 10000 });
		assert_ok!(EthereumControl::upgrade(
			origin,
			address,
			code_hash,
			initializer
		));
	});
}

#[test]
fn create_channel() {
	new_test_ext().execute_with(|| {
		let origin_para_id = 2000;
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(origin_para_id)) };
		let sovereign_account = ParaId::from(origin_para_id).into_account_truncating();
		let origin = make_xcm_origin(origin_location);

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 10000);

		assert_ok!(EthereumControl::create_agent(origin.clone()));
		assert_ok!(EthereumControl::create_channel(origin));
	});
}

#[test]
fn create_channel_fail_already_exists() {
	new_test_ext().execute_with(|| {
		let origin_para_id = 2000;
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(origin_para_id)) };
		let sovereign_account = ParaId::from(origin_para_id).into_account_truncating();
		let origin = make_xcm_origin(origin_location);

		// fund sovereign account of origin
		let _ = Balances::mint_into(&sovereign_account, 10000);

		assert_ok!(EthereumControl::create_agent(origin.clone()));
		assert_ok!(EthereumControl::create_channel(origin.clone()));

		assert_noop!(
			EthereumControl::create_channel(origin),
			Error::<Test>::ChannelAlreadyCreated
		);
	});
}

#[test]
fn create_channel_bad_origin() {
	new_test_ext().execute_with(|| {
		// relay chain location not allowed
		assert_noop!(
			EthereumControl::create_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 1,
						interior: Here,
					}
				)
			),
			BadOrigin,
		);

		// child of sibling location not allowed
		assert_noop!(
			EthereumControl::create_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}
				)
			),
			BadOrigin,
		);

		// local account location not allowed
		assert_noop!(
			EthereumControl::create_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 0,
						interior: X1(Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}
				)
			),
			BadOrigin,
		);

		// Signed origin not allowed
		assert_noop!(
			EthereumControl::create_channel(RuntimeOrigin::signed([14; 32].into())),
			BadOrigin
		);

		// None origin not allowed
		assert_noop!(
			EthereumControl::create_agent(RuntimeOrigin::none()),
			BadOrigin
		);
	});
}

#[test]
fn update_channel() {
	new_test_ext().execute_with(|| {
		let origin_para_id = 2000;
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(origin_para_id)) };
		let sovereign_account = ParaId::from(origin_para_id).into_account_truncating();
		let origin = make_xcm_origin(origin_location);

		// First create the channel
		let _ = Balances::mint_into(&sovereign_account, 10000);
		EthereumControl::create_agent(origin.clone()).unwrap();
		EthereumControl::create_channel(origin.clone()).unwrap();

		// Now try to update it
		assert_ok!(EthereumControl::update_channel(origin, OperatingMode::Normal, 2004));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::UpdateChannel {
			para_id: 2000.into(),
			mode: OperatingMode::Normal,
			fee: 2004,
		}));
	});
}

#[test]
fn update_channel_bad_origin() {
	new_test_ext().execute_with(|| {
		let mode = OperatingMode::Normal;
		let fee = 45;


		// relay chain location not allowed
		assert_noop!(
			EthereumControl::update_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 1,
						interior: Here,
					}
				),
				mode,
				fee,
			),
			BadOrigin,
		);

		// child of sibling location not allowed
		assert_noop!(
			EthereumControl::update_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}
				),
				mode,
				fee,
			),
			BadOrigin,
		);

		// local account location not allowed
		assert_noop!(
			EthereumControl::update_channel(
				make_xcm_origin(
					MultiLocation {
						parents: 0,
						interior: X1(Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}
				),
				mode,
				fee,
			),
			BadOrigin,
		);

		// Signed origin not allowed
		assert_noop!(
			EthereumControl::update_channel(RuntimeOrigin::signed([14; 32].into()), mode, fee),
			BadOrigin
		);

		// None origin not allowed
		assert_noop!(
			EthereumControl::update_channel(RuntimeOrigin::none(), mode, fee),
			BadOrigin
		);
	});
}

#[test]
fn update_channel_fails_not_exist() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(2000)) };
		let origin = make_xcm_origin(origin_location);

		// Now try to update it
		assert_noop!(
			EthereumControl::update_channel(origin, OperatingMode::Normal, 2004),
			Error::<Test>::ChannelNotExist
		);
	});
}

#[test]
fn force_update_channel() {
	new_test_ext().execute_with(|| {
		let origin_para_id = 2000;
		let origin_location = MultiLocation { parents: 1, interior: X1(Parachain(origin_para_id)) };
		let sovereign_account = ParaId::from(origin_para_id).into_account_truncating();
		let origin = make_xcm_origin(origin_location);

		// First create the channel
		let _ = Balances::mint_into(&sovereign_account, 10000);
		EthereumControl::create_agent(origin.clone()).unwrap();
		EthereumControl::create_channel(origin.clone()).unwrap();

		// Now try to force update it
		let force_origin = RuntimeOrigin::root();
		let versioned_location: Box<VersionedMultiLocation> = Box::new(origin_location.into());
		assert_ok!(EthereumControl::force_update_channel(force_origin, versioned_location, OperatingMode::Normal, 2004));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::UpdateChannel {
			para_id: 2000.into(),
			mode: OperatingMode::Normal,
			fee: 2004,
		}));
	});
}

#[test]
fn force_update_channel_bad_origin() {
	new_test_ext().execute_with(|| {
		let mode = OperatingMode::Normal;
		let fee = 45;

		// signed origin not allowed
		assert_noop!(
			EthereumControl::force_update_channel(
				RuntimeOrigin::signed([14; 32].into()),
				Box::new(
					MultiLocation {
						parents: 1,
						interior: Here,
					}.into()
				),
				mode,
				fee,
			),
			BadOrigin,
		);
	});
}

#[test]
fn force_update_channel_fail_invalid_location() {
	new_test_ext().execute_with(|| {
		let mode = OperatingMode::Normal;
		let fee = 45;

		// relay chain location not allowed
		assert_noop!(
			EthereumControl::force_update_channel(
				RuntimeOrigin::root(),
				Box::new(
					MultiLocation {
						parents: 1,
						interior: Here,
					}.into()
				),
				mode,
				fee,
			),
			Error::<Test>::InvalidLocation,
		);

		// local account location not allowed
		assert_noop!(
			EthereumControl::force_update_channel(
				RuntimeOrigin::root(),
				Box::new(
					MultiLocation {
						parents: 0,
						interior: X1(Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}.into()
				),
				mode,
				fee,
			),
			Error::<Test>::InvalidLocation,
		);

		// child of sibling location not allowed
		assert_noop!(
			EthereumControl::force_update_channel(
				RuntimeOrigin::root(),
				Box::new(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}.into()
				),
				mode,
				fee,
			),
			Error::<Test>::InvalidLocation,
		);
	});
}

#[test]
fn set_operating_mode_as_root() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let mode = OperatingMode::RejectingOutboundMessages;

		assert_ok!(EthereumControl::set_operating_mode(origin, mode));

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::SetOperatingMode {
			mode
		}));
	});
}

#[test]
fn set_operating_mode_as_signed_fails() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::signed([14; 32].into());
		let mode = OperatingMode::RejectingOutboundMessages;

		assert_noop!(EthereumControl::set_operating_mode(origin, mode), BadOrigin);
	});
}

#[test]
fn transfer_native_from_agent() {
	new_test_ext().execute_with(|| {
		let origin_location = MultiLocation {
			parents: 1,
			interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
		};
		let recipient: H160 = [27u8; 20].into();
		let amount = 103435;

		// First create the agent
		Agents::<Test>::insert(make_agent_id(origin_location), ());

		let origin = make_xcm_origin(origin_location);
		assert_ok!(
			EthereumControl::transfer_native_from_agent(origin, recipient, amount),
		);

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::TransferNativeFromAgent {
			agent_id: make_agent_id(origin_location),
			recipient,
			amount
		}));

	});
}

#[test]
fn force_transfer_native_from_agent() {
	new_test_ext().execute_with(|| {
		let origin = RuntimeOrigin::root();
		let location = MultiLocation {
			parents: 1,
			interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
		};
		let versioned_location: Box<VersionedMultiLocation> = Box::new(location.into());
		let recipient: H160 = [27u8; 20].into();
		let amount = 103435;

		// First create the agent
		Agents::<Test>::insert(make_agent_id(location), ());

		assert_ok!(
			EthereumControl::force_transfer_native_from_agent(origin, versioned_location, recipient, amount),
		);

		System::assert_last_event(RuntimeEvent::EthereumControl(crate::Event::TransferNativeFromAgent {
			agent_id: make_agent_id(location),
			recipient,
			amount
		}));

	});
}

#[test]
fn force_transfer_native_from_agent_bad_origin() {
	new_test_ext().execute_with(|| {
		let recipient: H160 = [27u8; 20].into();
		let amount = 103435;

		// signed origin not allowed
		assert_noop!(
			EthereumControl::force_transfer_native_from_agent(
				RuntimeOrigin::signed([14; 32].into()),
				Box::new(
					MultiLocation {
						parents: 1,
						interior: X2(Parachain(2000), Junction::AccountId32 { network: None, id: [67u8; 32]}),
					}.into()
				),
				recipient,
				amount,
			),
			BadOrigin,
		);
	});
}

// NOTE: The following tests are not actually tests and are more about obtaining location conversions
// for devops purposes. They need to be removed here and incorporated into a command line utility.

#[ignore]
#[test]
fn sibling_sovereign_account() {
	new_test_ext().execute_with(|| {
		let para_id = 1001;
		let sovereign_account: AccountId32 = ParaId::from(para_id).into_account_truncating();
		println!(
			"Sovereign account for parachain {}: {:#?}",
			para_id,
			hex::encode(sovereign_account)
		);

	});
}
