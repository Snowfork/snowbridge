// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use frame_support::{
	assert_err, assert_noop, assert_ok, parameter_types,
	traits::{ConstU32, Everything, Hooks, ProcessMessageError},
	weights::WeightMeter,
	BoundedBTreeMap,
};

use snowbridge_core::outbound::{AgentExecuteCommand, Command, CommandIndex};
use sp_core::{H160, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Keccak256},
	AccountId32, BoundedVec,
};
use sp_std::convert::From;
use xcm::prelude::Fungible;

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
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Hashing = Keccak256;
	type MessageQueue = MessageQueue;
	type MaxMessagePayloadSize = MaxMessagePayloadSize;
	type MaxMessagesPerBlock = MaxMessagesPerBlock;
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
					params: Some((0..100).map(|_| 1u8).collect::<Vec<u8>>()),
				},
			};

			let result = OutboundQueue::validate(&message);
			assert!(result.is_ok());
			let ticket = result.unwrap();

			assert_ok!(OutboundQueue::submit(ticket));
		}

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::CreateAgent { agent_id: Default::default() },
			};

			let result = OutboundQueue::validate(&message);
			assert!(result.is_ok());
			let ticket = result.unwrap();

			assert_ok!(OutboundQueue::submit(ticket));
		}

		for para_id in 1000..1004 {
			let message = Message {
				origin: para_id.into(),
				command: Command::Upgrade {
					impl_address: Default::default(),
					impl_code_hash: Default::default(),
					params: None,
				},
			};

			let result = OutboundQueue::validate(&message);
			assert!(result.is_ok());
			let ticket = result.unwrap();

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
				params: Some((0..1000).map(|_| 1u8).collect::<Vec<u8>>()),
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
		let message = (0..100).map(|_| 1u8).collect::<Vec<u8>>();
		let message: BoundedVec<u8, MaxEnqueuedMessageSizeOf<Test>> = message.try_into().unwrap();

		let mut meter = WeightMeter::max_limit();

		assert_noop!(
			OutboundQueue::process_message(
				&message.as_bounded_slice(),
				origin,
				&mut meter,
				&mut [0u8; 32]
			),
			ProcessMessageError::Yield
		);
	})
}

#[test]
fn process_message_fails_on_overweight_message() {
	new_tester().execute_with(|| {
		let origin = AggregateMessageOrigin::Parachain(1000.into());
		let message = (0..100).map(|_| 1u8).collect::<Vec<u8>>();
		let message: BoundedVec<u8, MaxEnqueuedMessageSizeOf<Test>> = message.try_into().unwrap();

		let mut meter = WeightMeter::from_limit(Weight::from_parts(1, 1));

		assert_noop!(
			OutboundQueue::process_message(
				&message.as_bounded_slice(),
				origin,
				&mut meter,
				&mut [0u8; 32]
			),
			ProcessMessageError::Overweight(<Test as Config>::WeightInfo::do_process_message())
		);
	})
}

#[test]
fn estimate_fee_should_work() {
	new_tester().execute_with(|| {
		let message = Message {
			origin: 1001.into(),
			command: Command::CreateAgent { agent_id: Default::default() },
		};
		let fees = OutboundQueue::estimate_fee(&message).unwrap();
		let fee_amount: u128 = match fees.get(0) {
			Some(&MultiAsset { fun: Fungible(amount), .. }) => amount,
			_ => 0,
		};
		assert_eq!(fee_amount, 19000000000);
	});
}

#[test]
fn compute_fee_reward_for_transfer_token() {
	new_tester().execute_with(|| {
		let message = Message {
			origin: 1001.into(),
			command: Command::AgentExecute {
				agent_id: Default::default(),
				command: AgentExecuteCommand::TransferToken {
					token: Default::default(),
					recipient: Default::default(),
					amount: 100,
				},
			},
		};
		let fees = OutboundQueue::compute_fee_reward(&message.command).unwrap();
		assert_eq!(fees.0, 1000000000);
		assert_eq!(fees.1, 337500000000000);
	});
}

#[test]
fn compute_fee_reward_for_upgrade() {
	new_tester().execute_with(|| {
		let message = Message {
			origin: 1001.into(),
			command: Command::Upgrade {
				impl_address: Default::default(),
				impl_code_hash: Default::default(),
				params: None,
			},
		};
		let fees = OutboundQueue::compute_fee_reward(&message.command).unwrap();
		assert_eq!(fees.0, 0);
		assert_eq!(fees.1, 5625000000000000);
	});
}

#[test]
fn set_outbound_fee_config_should_work() {
	new_tester().execute_with(|| {
		// estimate fee before reset command gas
		let message = Message {
			origin: 1001.into(),
			command: Command::CreateAgent { agent_id: Default::default() },
		};
		let fees = OutboundQueue::compute_fee_reward(&message.command).unwrap();
		assert_eq!(fees.0, 19000000000);
		assert_eq!(fees.1, 3375000000000000);

		let mut command_gas_map = BoundedBTreeMap::<
			CommandIndex,
			GasAmount,
			ConstU32<{ CommandIndex::max_value() as u32 }>,
		>::new();
		// 2 is the command index of create_agent
		command_gas_map.try_insert(2_u8, 500000).unwrap();
		let mut config = OutboundFeeConfig::default();
		config.command_gas_map = Some(command_gas_map);
		let origin = RuntimeOrigin::root();
		assert_ok!(OutboundQueue::set_outbound_fee_config(origin, config));

		// estimate fee after reset command gas
		let fees = OutboundQueue::compute_fee_reward(&message.command).unwrap();
		assert_eq!(fees.0, 31000000000);
		assert_eq!(fees.1, 5625000000000000);
	});
}
