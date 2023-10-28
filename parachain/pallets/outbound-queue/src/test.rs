// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use crate::{mock::*, *};

use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::{Hooks, ProcessMessage, ProcessMessageError},
	weights::WeightMeter,
};

use codec::Encode;
use snowbridge_core::outbound::{
	AgentExecuteCommand, Command, ExportOrigin, Initializer, Message, SendError, SendMessage,
};
use sp_core::{H160, H256};
use sp_runtime::{AccountId32, DispatchError};

#[test]
fn submit_messages_from_multiple_origins_and_commit() {
	new_tester().execute_with(|| {
		//next_block();

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::Upgrade {
					impl_address: H160::zero(),
					impl_code_hash: H256::zero(),
					initializer: None,
				},
			};

			let (ticket, _) = OutboundQueue::validate(&message).unwrap();
			assert_ok!(OutboundQueue::deliver(ticket));
		}

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::CreateAgent { agent_id: Default::default() },
			};

			let (ticket, _) = OutboundQueue::validate(&message).unwrap();
			assert_ok!(OutboundQueue::deliver(ticket));
		}

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::Upgrade {
					impl_address: Default::default(),
					impl_code_hash: Default::default(),
					initializer: None,
				},
			};

			let (ticket, _) = OutboundQueue::validate(&message).unwrap();

			assert_ok!(OutboundQueue::deliver(ticket));
		}

		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();

		for para_id in 1000..1004 {
			let origin: ParaId = (para_id as u32).into();
			assert_eq!(Nonce::<Test>::get(origin), 3);
		}

		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
	});
}

#[test]
fn submit_message_fail_too_large() {
	new_tester().execute_with(|| {
		let message = Message {
			origin: 1000.into(),
			command: Command::Upgrade {
				impl_address: H160::zero(),
				impl_code_hash: H256::zero(),
				initializer: Some(Initializer {
					params: (0..1000).map(|_| 1u8).collect::<Vec<u8>>(),
					maximum_required_gas: 0,
				}),
			},
		};

		assert_err!(OutboundQueue::validate(&message), SendError::MessageTooLarge);
	});
}

#[test]
fn calculate_fees() {
	new_tester().execute_with(|| {
		let command = Command::Upgrade {
			impl_address: H160::zero(),
			impl_code_hash: H256::zero(),
			initializer: Some(Initializer {
				params: (0..256).map(|_| 1u8).collect::<Vec<u8>>(),
				maximum_required_gas: 0,
			}),
		};

		let fee = OutboundQueue::calculate_fee(&command).unwrap();

		// ((gas(2) * fee_per_gas(1)) + reward(1)) / xrate(1/10) = 30
		assert_eq!(fee.remote, 30);
	});
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
			OutboundQueue::set_fee_config(origin, Default::default()),
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
fn submit_low_priority_messages_yield_when_there_is_high_priority_message() {
	new_tester().execute_with(|| {
		// submit a low priority message from asset_hub first
		let message = Message {
			origin: 1000.into(),
			command: Command::AgentExecute {
				agent_id: Default::default(),
				command: AgentExecuteCommand::TransferToken {
					token: Default::default(),
					recipient: Default::default(),
					amount: 0,
				},
			},
		};
		let result = OutboundQueue::validate(&message);
		assert!(result.is_ok());
		let ticket = result.unwrap();
		assert_ok!(OutboundQueue::deliver(ticket.0));

		// then submit a high priority message from bridge_hub
		let message = Message {
			origin: 1013.into(),
			command: Command::Upgrade {
				impl_address: H160::zero(),
				impl_code_hash: H256::zero(),
				initializer: None,
			},
		};
		let result = OutboundQueue::validate(&message);
		assert!(result.is_ok());
		let ticket = result.unwrap();
		assert_ok!(OutboundQueue::deliver(ticket.0));
		let mut footprint =
			MessageQueue::footprint(AggregateMessageOrigin::Export(ExportOrigin::Here));
		println!("{:?}", footprint);
		assert_eq!(footprint.count, 1);

		// process a low priority message from asset_hub will yield
		let origin = AggregateMessageOrigin::Export(ExportOrigin::Sibling(1000.into()));
		let message = QueuedMessage {
			id: Default::default(),
			origin: 1000.into(),
			command: Command::AgentExecute {
				agent_id: Default::default(),
				command: AgentExecuteCommand::TransferToken {
					token: Default::default(),
					recipient: Default::default(),
					amount: 0,
				},
			},
		}
		.encode();

		let mut meter = WeightMeter::new();

		assert_noop!(
			OutboundQueue::process_message(&message.as_slice(), origin, &mut meter, &mut [0u8; 32]),
			ProcessMessageError::Yield
		);

		// run to next block and ensure high priority message processed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
		footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(ExportOrigin::Here));
		assert_eq!(footprint.count, 0);
	});
}

#[test]
fn submit_high_priority_message_will_not_blocked_even_when_low_priority_queue_get_spammed() {
	new_tester().execute_with(|| {
		// submit a lot of low priority messages from asset_hub which will need multiple blocks to
		// execute(60 for 3 blocks)
		let max_messages = 60;
		for _ in 0..max_messages {
			let message = Message {
				origin: 1000.into(),
				command: Command::AgentExecute {
					agent_id: Default::default(),
					command: AgentExecuteCommand::TransferToken {
						token: Default::default(),
						recipient: Default::default(),
						amount: 0,
					},
				},
			};
			let result = OutboundQueue::validate(&message);
			assert!(result.is_ok());
			let ticket = result.unwrap();
			assert_ok!(OutboundQueue::deliver(ticket.0));
		}

		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(
			ExportOrigin::Sibling(1000.into()),
		));
		assert_eq!(footprint.count, (max_messages) as u64);

		// submit high priority message from bridge_hub
		let message = Message {
			origin: 1013.into(),
			command: Command::Upgrade {
				impl_address: H160::zero(),
				impl_code_hash: H256::zero(),
				initializer: None,
			},
		};
		let result = OutboundQueue::validate(&message);
		assert!(result.is_ok());
		let ticket = result.unwrap();
		assert_ok!(OutboundQueue::deliver(ticket.0));
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(ExportOrigin::Here));
		println!("{:?}", footprint);
		assert_eq!(footprint.count, 1);

		// run to next block high priority message and some of the low priority messages get
		// executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(
			ExportOrigin::Sibling(1000.into()),
		));
		assert_eq!(footprint.count, 41);
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(ExportOrigin::Here));
		assert_eq!(footprint.count, 0);

		// move to the next block, some low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(
			ExportOrigin::Sibling(1000.into()),
		));
		assert_eq!(footprint.count, 21);

		// move to the next block, some low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(
			ExportOrigin::Sibling(1000.into()),
		));
		assert_eq!(footprint.count, 1);

		// move to the next block, the last low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Export(
			ExportOrigin::Sibling(1000.into()),
		));
		assert_eq!(footprint.count, 0);
	});
}

// Governance messages should be able to bypass a halted operating mode
// Other message sends should fail when halted
#[test]
fn submit_upgrade_message_success_when_queue_halted() {
	new_tester().execute_with(|| {
		// halt the outbound queue
		assert_ok!(OutboundQueue::set_operating_mode(
			RuntimeOrigin::root(),
			BasicOperatingMode::Halted
		));

		// submit a high priority message from bridge_hub should success
		let message = Message {
			origin: 1013.into(),
			command: Command::Upgrade {
				impl_address: H160::zero(),
				impl_code_hash: H256::zero(),
				initializer: None,
			},
		};
		let result = OutboundQueue::validate(&message);
		assert!(result.is_ok());
		let ticket = result.unwrap();
		assert_ok!(OutboundQueue::deliver(ticket.0));

		// submit a low priority message from asset_hub will fail
		let message = Message {
			origin: 1000.into(),
			command: Command::AgentExecute {
				agent_id: Default::default(),
				command: AgentExecuteCommand::TransferToken {
					token: Default::default(),
					recipient: Default::default(),
					amount: 0,
				},
			},
		};
		let result = OutboundQueue::validate(&message);
		assert!(result.is_ok());
		let ticket = result.unwrap();
		assert_noop!(OutboundQueue::deliver(ticket.0), SendError::Halted);
	});
}
