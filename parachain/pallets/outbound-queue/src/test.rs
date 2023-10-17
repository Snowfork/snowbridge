use bp_runtime::BasicOperatingMode;
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use frame_support::{
	assert_err, assert_noop, assert_ok, parameter_types,
	traits::{Everything, Hooks, ProcessMessageError},
	weights::WeightMeter,
};

use snowbridge_core::outbound::{AgentExecuteCommand, Command, Initializer};
use sp_core::{ConstU128, H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Keccak256},
	AccountId32,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type AccountId = AccountId32;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
		MessageQueue: pallet_message_queue::{Pallet, Call, Storage, Event<T>},
		OutboundQueue: crate::{Pallet, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const HeapSize: u32 = 32 * 1024;
	pub const MaxStale: u32 = 32;
	pub static ServiceWeight: Option<Weight> = Some(Weight::from_parts(100, 100));
}

impl pallet_message_queue::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MessageProcessor = OutboundQueue;
	type Size = u32;
	type QueueChangeHandler = ();
	type HeapSize = HeapSize;
	type MaxStale = MaxStale;
	type ServiceWeight = ServiceWeight;
}

parameter_types! {
	pub const MaxMessagePayloadSize: u32 = 1024;
	pub const MaxMessagesPerBlock: u32 = 20;
	pub const OwnParaId: ParaId = ParaId::new(1013);
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type MessageQueue = MessageQueue;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerBlock = MaxMessagesPerBlock;
	type OwnParaId = OwnParaId;
	type GasMeter = ();
	type Balance = u128;
	type DeliveryFeePerGas = ConstU128<1>;
	type DeliveryRefundPerGas = ConstU128<1>;
	type DeliveryReward = ConstU128<1>;
	type WeightInfo = ();
}

fn setup() {
	System::set_block_number(1);
}

pub fn new_tester() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| setup());
	ext
}

fn run_to_end_of_next_block() {
	// finish current block
	MessageQueue::on_finalize(System::block_number());
	OutboundQueue::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
	// start next block
	System::set_block_number(System::block_number() + 1);
	System::on_initialize(System::block_number());
	OutboundQueue::on_initialize(System::block_number());
	MessageQueue::on_initialize(System::block_number());
	// finish next block
	MessageQueue::on_finalize(System::block_number());
	OutboundQueue::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
}

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
			assert_ok!(OutboundQueue::submit(ticket));
		}

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::CreateAgent { agent_id: Default::default() },
			};

			let (ticket, _) = OutboundQueue::validate(&message).unwrap();
			assert_ok!(OutboundQueue::submit(ticket));
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

			assert_ok!(OutboundQueue::submit(ticket));
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

		assert_err!(OutboundQueue::validate(&message), SubmitError::MessageTooLarge);
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

		let origin = AggregateMessageOrigin::Parachain(1000.into());
		let message = EnqueuedMessage {
			id: Default::default(),
			origin: 1000.into(),
			command: Command::Upgrade {
				impl_address: Default::default(),
				impl_code_hash: Default::default(),
				initializer: None,
			},
		}
		.encode();

		let mut meter = WeightMeter::max_limit();

		assert_noop!(
			OutboundQueue::process_message(&message.as_slice(), origin, &mut meter, &mut [0u8; 32]),
			ProcessMessageError::Yield
		);
	})
}

#[test]
fn process_message_fails_on_overweight_message() {
	new_tester().execute_with(|| {
		let origin = AggregateMessageOrigin::Parachain(1000.into());

		let message = EnqueuedMessage {
			id: Default::default(),
			origin: 1000.into(),
			command: Command::Upgrade {
				impl_address: Default::default(),
				impl_code_hash: Default::default(),
				initializer: None,
			},
		}
		.encode();

		let mut meter = WeightMeter::from_limit(Weight::from_parts(1, 1));

		assert_noop!(
			OutboundQueue::process_message(&message.as_slice(), origin, &mut meter, &mut [0u8; 32]),
			ProcessMessageError::Overweight(<Test as Config>::WeightInfo::do_process_message())
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
		assert_ok!(OutboundQueue::submit(ticket.0));

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
		assert_ok!(OutboundQueue::submit(ticket.0));
		let mut footprint =
			MessageQueue::footprint(AggregateMessageOrigin::SelfChain(Priority::High));
		println!("{:?}", footprint);
		assert_eq!(footprint.count, 1);

		// process a low priority message from asset_hub will yield
		let origin = AggregateMessageOrigin::Parachain(1000.into());
		let message = EnqueuedMessage {
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

		let mut meter = WeightMeter::max_limit();

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
		footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1013.into()));
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
			assert_ok!(OutboundQueue::submit(ticket.0));
		}

		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1000.into()));
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
		assert_ok!(OutboundQueue::submit(ticket.0));
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::SelfChain(Priority::High));
		println!("{:?}", footprint);
		assert_eq!(footprint.count, 1);

		// run to next block high priority message and some of the low priority messages get
		// executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let digest = System::digest();
		let digest_items = digest.logs();
		assert!(digest_items.len() == 1 && digest_items[0].as_other().is_some());
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1000.into()));
		assert_eq!(footprint.count, 41);
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::SelfChain(Priority::High));
		assert_eq!(footprint.count, 0);

		// move to the next block, some low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1000.into()));
		assert_eq!(footprint.count, 21);

		// move to the next block, some low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1000.into()));
		assert_eq!(footprint.count, 1);

		// move to the next block, the last low priority messages get executed
		ServiceWeight::set(Some(Weight::MAX));
		run_to_end_of_next_block();
		let footprint = MessageQueue::footprint(AggregateMessageOrigin::Parachain(1000.into()));
		assert_eq!(footprint.count, 0);
	});
}

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
		assert_ok!(OutboundQueue::submit(ticket.0));

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
		assert_noop!(OutboundQueue::submit(ticket.0), SubmitError::BridgeHalted);
	});
}
