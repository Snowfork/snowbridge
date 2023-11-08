// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};

use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::{Hooks, ProcessMessage, ProcessMessageError},
	weights::WeightMeter,
};

use codec::Encode;
use snowbridge_core::outbound::{Command, ExportOrigin, SendError, SendMessage};
use sp_core::H256;
use sp_runtime::{AccountId32, DispatchError};

#[test]
fn submit_messages_and_commit() {
	new_tester().execute_with(|| {
		for para_id in 1000..1004 {
			let message = mock_message(para_id);
			let (ticket, _) = OutboundQueue::validate(&message).unwrap();
			assert_ok!(OutboundQueue::deliver(ticket));
		}

		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();

		for para_id in 1000..1004 {
			let origin: ParaId = (para_id as u32).into();
			assert_eq!(Nonce::<Test>::get(origin), 1);
		}

		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
		assert_eq!(Messages::<Test>::decode_len(), Some(4));
	});
}

#[test]
fn submit_message_fail_too_large() {
	new_tester().execute_with(|| {
		let message = mock_invalid_governance_message::<Test>();
		assert_err!(OutboundQueue::validate(&message), SendError::MessageTooLarge);
	});
}

#[test]
fn calculate_fees() {
	new_tester().execute_with(|| {
		let command = mock_message(1000).command;
		let fee = OutboundQueue::calculate_fee(&command);
		assert_eq!(fee.remote, 2200000000000);

		println!("Total fee: {}", fee.total())
	});
}

#[test]
fn convert_from_ether_decimals() {
	assert_eq!(
		OutboundQueue::convert_from_ether_decimals(1_000_000_000_000_000_000),
		100_000_000_000_0
	);
}

#[test]
fn commit_exits_early_if_no_processed_messages() {
	new_tester().execute_with(|| {
		// on_finalize should do nothing, nor should it panic
		OutboundQueue::on_finalize(System::block_number());

		let digest = System::digest();
		let digest_items = digest.logs();
		assert_eq!(digest_items.len(), 0);
	});
}

#[test]
fn process_message_yields_on_max_messages_per_block() {
	new_tester().execute_with(|| {
		for _ in 0..<Test as Config>::MaxMessagesPerBlock::get() {
			MessageLeaves::<Test>::append(H256::zero())
		}

		let origin = AggregateMessageOrigin::Export(ExportOrigin::Sibling(1000.into()));
		let message = QueuedMessage {
			id: Default::default(),
			origin: 1000.into(),
			command: Command::Upgrade {
				impl_address: Default::default(),
				impl_code_hash: Default::default(),
				initializer: None,
			},
		}
		.encode();

		let mut meter = WeightMeter::new();

		assert_noop!(
			OutboundQueue::process_message(&message.as_slice(), origin, &mut meter, &mut [0u8; 32]),
			ProcessMessageError::Yield
		);
	})
}

#[test]
fn process_message_fails_on_overweight_message() {
	new_tester().execute_with(|| {
		let sibling_id = 1000;
		let origin = AggregateMessageOrigin::Export(ExportOrigin::Sibling(sibling_id.into()));
		let message = mock_message(sibling_id).encode();
		let mut meter = WeightMeter::with_limit(Weight::from_parts(1, 1));
		assert_noop!(
			OutboundQueue::process_message(&message.as_slice(), origin, &mut meter, &mut [0u8; 32]),
			ProcessMessageError::Overweight(<Test as Config>::WeightInfo::do_process_message())
		);
	})
}

#[test]
fn set_operating_mode_root_only() {
	new_tester().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::from([0; 32]));
		assert_noop!(
			OutboundQueue::set_operating_mode(origin, BasicOperatingMode::Halted),
			DispatchError::BadOrigin,
		);
	})
}

#[test]
fn set_fee_config_root_only() {
	new_tester().execute_with(|| {
		let origin = RuntimeOrigin::signed(AccountId32::from([0; 32]));
		assert_noop!(
			OutboundQueue::set_fee_config(origin, DefaultFeeConfig::get()),
			DispatchError::BadOrigin,
		);
	})
}

#[test]
fn set_fee_config_invalid() {
	new_tester().execute_with(|| {
		let origin = RuntimeOrigin::root();
		assert_noop!(
			OutboundQueue::set_fee_config(
				origin,
				FeeConfigRecord { exchange_rate: (1, 1).into(), reward: 0, fee_per_gas: 0 }
			),
			Error::<Test>::InvalidFeeConfig
		);
	})
}

#[test]
fn low_priority_messages_are_processed_last() {
	use AggregateMessageOrigin::*;
	use ExportOrigin::*;

	let sibling_id = 1000;
	let high_priority_queue = Export(Here);
	let low_priority_queue = Export(Sibling(sibling_id.into()));

	new_tester().execute_with(|| {
		// submit a lot of high priority messages from asset_hub which will need multiple blocks to
		// execute(60 for 3 blocks)
		let max_messages = 60;
		for _ in 0..max_messages {
			let message = mock_governance_message::<Test>();
			let (ticket, _) = OutboundQueue::validate(&message).unwrap();
			OutboundQueue::deliver(ticket).unwrap();
		}
		let footprint = MessageQueue::footprint(high_priority_queue);
		assert_eq!(footprint.count, (max_messages) as u64);

		// submit low priority message
		let message = mock_message(sibling_id);
		let (ticket, _) = OutboundQueue::validate(&message).unwrap();
		OutboundQueue::deliver(ticket).unwrap();
		let footprint = MessageQueue::footprint(low_priority_queue);
		assert_eq!(footprint.count, 1);

		// run to next block; only high priority messages should have been processed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(high_priority_queue);
		assert_eq!(footprint.count, 40);

		let footprint = MessageQueue::footprint(low_priority_queue);
		assert_eq!(footprint.count, 1);

		// move to the next block, some high priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(high_priority_queue);
		assert_eq!(footprint.count, 20);

		let footprint = MessageQueue::footprint(low_priority_queue);
		assert_eq!(footprint.count, 1);

		// move to the next block, some high priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(high_priority_queue);
		assert_eq!(footprint.count, 0);

		let footprint = MessageQueue::footprint(low_priority_queue);
		assert_eq!(footprint.count, 1);

		// move to the next block, the last remaining pending message,
		// a lower priority one, is processed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();

		let footprint = MessageQueue::footprint(low_priority_queue);
		assert_eq!(footprint.count, 0);
	});
}

// Governance messages should be able to bypass a halted operating mode
// Other message sends should fail when halted
#[test]
fn submit_upgrade_message_success_when_queue_halted() {
	new_tester().execute_with(|| {
		// halt the outbound queue
		OutboundQueue::set_operating_mode(RuntimeOrigin::root(), BasicOperatingMode::Halted)
			.unwrap();

		// submit a high priority message from bridge_hub should success
		let message = mock_governance_message::<Test>();
		let (ticket, _) = OutboundQueue::validate(&message).unwrap();
		assert_ok!(OutboundQueue::deliver(ticket));

		// submit a low priority message from asset_hub will fail as pallet is halted
		let message = mock_message(1000);
		let (ticket, _) = OutboundQueue::validate(&message).unwrap();
		assert_noop!(OutboundQueue::deliver(ticket), SendError::Halted);
	});
}
