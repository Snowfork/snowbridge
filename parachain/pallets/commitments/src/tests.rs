// Copyright 2020 Parity Technologies (UK) Ltd.
use crate::{mock::*};

use crate::{Message, MessageQueue};

use sp_runtime::DigestItem;
use sp_core::H160;

use frame_support::{
	traits::{OnInitialize}
};

use frame_support::storage::StorageValue;

use artemis_core::Commitments;

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		CommitmentsModule::on_initialize(System::block_number());
	}
}


const CONTRACT_A: H160 =  H160::repeat_byte(1);
const CONTRACT_B: H160 =  H160::repeat_byte(2);


#[test]
fn test_add_message() {
	new_test_ext().execute_with(|| {
		CommitmentsModule::add(CONTRACT_A, vec![0, 1, 2]);
		CommitmentsModule::add(CONTRACT_B, vec![3, 4, 5]);

		let messages = vec![
			Message {
				address: CONTRACT_A,
				payload: vec![0, 1, 2],
				nonce: 0
			},
			Message {
				address: CONTRACT_B,
				payload: vec![3, 4, 5],
				nonce: 1
			},
		];

		assert_eq!(
			MessageQueue::get(), messages);
		assert_eq!(CommitmentsModule::nonce(), 2);

		// Run to block 5 where a commitment will be generated
		run_to_block(5);

		assert_eq!(
			MessageQueue::exists(), false
		);
		assert_eq!(
			System::digest().logs(),
			vec![
				DigestItem::Other(vec![0, 48, 89, 246, 187, 20, 156, 87, 142, 138, 90, 46, 234, 197, 120, 204, 50, 208, 209, 63, 125, 48, 204, 124, 195, 132, 234, 48, 140, 24, 59, 6, 244])
			]
		);

	});
}
