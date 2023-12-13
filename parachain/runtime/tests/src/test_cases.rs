// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

//! Module contains predefined test-case scenarios for `Runtime` with bridging capabilities.

use asset_hub_rococo_runtime::xcm_config::bridging::to_ethereum::BridgeHubEthereumBaseFeeInROC;
use codec::Encode;
use frame_support::{
	assert_err, assert_ok,
	traits::{fungible::Mutate, OriginTrait},
};
use parachains_common::AccountId;
pub use parachains_runtimes_test_utils::test_cases::change_storage_constant_by_governance_works;
use parachains_runtimes_test_utils::{
	AccountIdOf, BalanceOf, CollatorSessionKeys, ExtBuilder, ValidatorIdOf, XcmReceivedFrom,
};
use snowbridge_core::{Channel, ChannelId, ParaId};
use snowbridge_inbound_queue::{
	fixtures::{make_create_message, InboundQueueTest},
	BenchmarkHelper,
};
use sp_core::H160;
use sp_runtime::SaturatedConversion;
use xcm::{
	latest::prelude::*,
	v3::Error::{Barrier, FailedToTransactAsset, NotHoldingFees},
};
use xcm_executor::XcmExecutor;

type RuntimeHelper<Runtime, AllPalletsWithoutSystem = ()> =
	parachains_runtimes_test_utils::RuntimeHelper<Runtime, AllPalletsWithoutSystem>;

pub fn initial_fund<Runtime>(assethub_parachain_id: u32, initial_amount: u128)
where
	Runtime: frame_system::Config + pallet_balances::Config,
{
	// fund asset hub sovereign account enough so it can pay fees
	let asset_hub_sovereign_account =
		snowbridge_core::sibling_sovereign_account::<Runtime>(assethub_parachain_id.into());
	<pallet_balances::Pallet<Runtime>>::mint_into(
		&asset_hub_sovereign_account,
		initial_amount.saturated_into::<BalanceOf<Runtime>>(),
	)
	.unwrap();
}

pub fn send_transfer_token_message<Runtime, XcmConfig>(
	assethub_parachain_id: u32,
	weth_contract_address: H160,
	destination_address: H160,
	fee_amount: u128,
) -> Outcome
where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config,
	XcmConfig: xcm_executor::Config,
{
	let assethub_parachain_location = MultiLocation::new(1, Parachain(assethub_parachain_id));
	let asset = MultiAsset {
		id: Concrete(MultiLocation {
			parents: 0,
			interior: X1(AccountKey20 { network: None, key: weth_contract_address.into() }),
		}),
		fun: Fungible(1000000000),
	};
	let assets = vec![asset.clone()];

	let inner_xcm = Xcm(vec![
		WithdrawAsset(MultiAssets::from(assets.clone())),
		ClearOrigin,
		BuyExecution { fees: asset, weight_limit: Unlimited },
		DepositAsset {
			assets: Wild(All),
			beneficiary: MultiLocation {
				parents: 0,
				interior: X1(AccountKey20 { network: None, key: destination_address.into() }),
			},
		},
		SetTopic([0; 32]),
	]);

	let fee = MultiAsset {
		id: Concrete(MultiLocation { parents: 1, interior: Here }),
		fun: Fungible(fee_amount),
	};

	// prepare transfer token message
	let xcm = Xcm(vec![
		WithdrawAsset(MultiAssets::from(vec![fee.clone()])),
		BuyExecution { fees: fee, weight_limit: Unlimited },
		ExportMessage {
			network: Ethereum { chain_id: 11155111 },
			destination: Here,
			xcm: inner_xcm,
		},
	]);

	// execute XCM
	let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
	XcmExecutor::<XcmConfig>::execute_xcm(
		assethub_parachain_location,
		xcm,
		hash,
		RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Sibling),
	)
}

pub fn send_transfer_token_message_success<Runtime, XcmConfig>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_para_id: u32,
	assethub_parachain_id: u32,
	weth_contract_address: H160,
	destination_address: H160,
	fee_amount: u128,
	snowbridge_outbound_queue: Box<
		dyn Fn(Vec<u8>) -> Option<snowbridge_outbound_queue::Event<Runtime>>,
	>,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config,
	XcmConfig: xcm_executor::Config,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			// fund asset hub sovereign account enough so it can pay fees
			initial_fund::<Runtime>(
				assethub_parachain_id,
				BridgeHubEthereumBaseFeeInROC::get() + 1_000_000_000,
			);

			let outcome = send_transfer_token_message::<Runtime, XcmConfig>(
				assethub_parachain_id,
				weth_contract_address,
				destination_address,
				fee_amount,
			);

			assert_ok!(outcome.ensure_complete());

			// check events
			let mut events = <frame_system::Pallet<Runtime>>::events()
				.into_iter()
				.filter_map(|e| snowbridge_outbound_queue(e.event.encode()));
			assert!(
				events.any(|e| matches!(e, snowbridge_outbound_queue::Event::MessageQueued { .. }))
			);
		});
}

pub fn send_unpaid_transfer_token_message<Runtime, XcmConfig>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_para_id: u32,
	assethub_parachain_id: u32,
	weth_contract_address: H160,
	destination_contract: H160,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config,
	XcmConfig: xcm_executor::Config,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
{
	let assethub_parachain_location = MultiLocation::new(1, Parachain(assethub_parachain_id));

	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			let asset_hub_sovereign_account =
				snowbridge_core::sibling_sovereign_account::<Runtime>(assethub_parachain_id.into());

			<pallet_balances::Pallet<Runtime>>::mint_into(
				&asset_hub_sovereign_account,
				4000000000u32.into(),
			)
			.unwrap();

			let asset = MultiAsset {
				id: Concrete(MultiLocation {
					parents: 0,
					interior: X1(AccountKey20 { network: None, key: weth_contract_address.into() }),
				}),
				fun: Fungible(1000000000),
			};
			let assets = vec![asset.clone()];

			let inner_xcm = Xcm(vec![
				WithdrawAsset(MultiAssets::from(assets.clone())),
				ClearOrigin,
				BuyExecution { fees: asset, weight_limit: Unlimited },
				DepositAsset {
					assets: Wild(AllCounted(1)),
					beneficiary: MultiLocation {
						parents: 0,
						interior: X1(AccountKey20 {
							network: None,
							key: destination_contract.into(),
						}),
					},
				},
				SetTopic([0; 32]),
			]);

			// prepare transfer token message
			let xcm = Xcm(vec![
				UnpaidExecution { weight_limit: Unlimited, check_origin: None },
				ExportMessage {
					network: Ethereum { chain_id: 11155111 },
					destination: Here,
					xcm: inner_xcm,
				},
			]);

			// execute XCM
			let hash = xcm.using_encoded(sp_io::hashing::blake2_256);
			let outcome = XcmExecutor::<XcmConfig>::execute_xcm(
				assethub_parachain_location,
				xcm,
				hash,
				RuntimeHelper::<Runtime>::xcm_max_weight(XcmReceivedFrom::Sibling),
			);
			// check error is barrier
			assert_err!(outcome.ensure_complete(), Barrier);
		});
}

pub fn send_transfer_token_message_fee_not_enough<Runtime, XcmConfig>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_para_id: u32,
	assethub_parachain_id: u32,
	weth_contract_address: H160,
	destination_address: H160,
	fee_amount: u128,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config,
	XcmConfig: xcm_executor::Config,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			// fund asset hub sovereign account enough so it can pay fees
			initial_fund::<Runtime>(
				assethub_parachain_id,
				BridgeHubEthereumBaseFeeInROC::get() + 1_000_000_000,
			);

			let outcome = send_transfer_token_message::<Runtime, XcmConfig>(
				assethub_parachain_id,
				weth_contract_address,
				destination_address,
				fee_amount,
			);
			// check err is NotHoldingFees
			assert_err!(outcome.ensure_complete(), NotHoldingFees);
		});
}

pub fn send_transfer_token_message_insufficient_fund<Runtime, XcmConfig>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_para_id: u32,
	assethub_parachain_id: u32,
	weth_contract_address: H160,
	destination_address: H160,
	fee_amount: u128,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config,
	XcmConfig: xcm_executor::Config,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			// initial fund not enough
			initial_fund::<Runtime>(assethub_parachain_id, 1_000_000_000);

			let outcome = send_transfer_token_message::<Runtime, XcmConfig>(
				assethub_parachain_id,
				weth_contract_address,
				destination_address,
				fee_amount,
			);
			// check err is FailedToTransactAsset("InsufficientBalance")
			assert_err!(outcome.ensure_complete(), FailedToTransactAsset("InsufficientBalance"));
		});
}

pub fn initialize_beacon<Runtime>(fixture: InboundQueueTest)
where
	Runtime: snowbridge_inbound_queue::Config,
{
	Runtime::Helper::initialize_storage(fixture.message.proof.block_hash, fixture.execution_header);
}

pub fn initialize_channel<Runtime>(assethub_parachain_id: u32)
where
	Runtime: snowbridge_system::Config,
{
	// Asset Hub
	let asset_hub_location: MultiLocation =
		ParentThen(X1(Parachain(assethub_parachain_id.into()))).into();
	let asset_hub_agent_id =
		snowbridge_system::agent_id_of::<Runtime>(&asset_hub_location).unwrap();

	let asset_hub_channel_id: ChannelId = (ParaId::from(assethub_parachain_id)).into();
	snowbridge_system::Agents::<Runtime>::insert(asset_hub_agent_id, ());
	snowbridge_system::Channels::<Runtime>::insert(
		asset_hub_channel_id,
		Channel { agent_id: asset_hub_agent_id, para_id: assethub_parachain_id.into() },
	);
}

pub fn submit_register_token_message_from_ethereum_works<Runtime>(
	collator_session_key: CollatorSessionKeys<Runtime>,
	runtime_para_id: u32,
	assethub_parachain_id: u32,
	snowbridge_inbound_queue_event: Box<
		dyn Fn(Vec<u8>) -> Option<snowbridge_inbound_queue::Event<Runtime>>,
	>,
) where
	Runtime: frame_system::Config
		+ pallet_balances::Config
		+ pallet_session::Config
		+ pallet_xcm::Config
		+ parachain_info::Config
		+ pallet_collator_selection::Config
		+ cumulus_pallet_parachain_system::Config
		+ snowbridge_outbound_queue::Config
		+ snowbridge_inbound_queue::Config
		+ snowbridge_system::Config,
	ValidatorIdOf<Runtime>: From<AccountIdOf<Runtime>>,
	<Runtime as frame_system::Config>::AccountId: From<sp_runtime::AccountId32>,
{
	ExtBuilder::<Runtime>::default()
		.with_collators(collator_session_key.collators())
		.with_session_keys(collator_session_key.session_keys())
		.with_para_id(runtime_para_id.into())
		.with_tracing()
		.build()
		.execute_with(|| {
			initial_fund::<Runtime>(assethub_parachain_id, 1_000_000_000_000_000);
			initialize_beacon::<Runtime>(make_create_message());
			initialize_channel::<Runtime>(assethub_parachain_id);

			let message = make_create_message().message;
			log::info!("{:?}", message);

			assert_ok!(<snowbridge_inbound_queue::Pallet<Runtime>>::submit(
				<Runtime as frame_system::Config>::RuntimeOrigin::signed(
					AccountId::from(crate::Alice).into()
				),
				message.clone()
			));
			let mut events = <frame_system::Pallet<Runtime>>::events()
				.into_iter()
				.filter_map(|e| snowbridge_inbound_queue_event(e.event.encode()));
			assert!(events.any(|e| {
				log::info!("{:?}", e);
				matches!(
					e,
					snowbridge_inbound_queue::Event::MessageReceived {
						message_id: [
							197, 79, 50, 255, 200, 228, 214, 3, 10, 230, 237, 56, 63, 84, 34, 43,
							140, 50, 176, 220, 7, 95, 137, 251, 237, 226, 101, 161, 227, 37, 156,
							134
						],
						..
					}
				)
			}));
		});
}
