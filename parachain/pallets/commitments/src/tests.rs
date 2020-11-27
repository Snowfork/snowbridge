// Copyright 2020 Parity Technologies (UK) Ltd.
use crate::{mock::*};

use crate::{Message, Messages};

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

#[test]
fn test_add_message() {
	new_test_ext().execute_with(|| {
		CommitmentsModule::add(H160::zero(), vec![0, 1, 2]);
		CommitmentsModule::add(H160::zero(), vec![3, 4, 5]);

		assert_eq!(
			Messages::get(),
			vec![
				Message {
					address: H160::zero(),
					payload: vec![0, 1, 2],
					nonce: 0
				},
				Message {
					address: H160::zero(),
					payload: vec![3, 4, 5],
					nonce: 1
				},
			]
		);
		assert_eq!(CommitmentsModule::nonce(), 2);

		run_to_block(5);

		assert_eq!(
			Messages::exists(), false
		);
		assert_eq!(
			System::digest().logs(),
			vec![
				DigestItem::Other(
					vec![0, 86, 198, 168, 58, 18, 129, 83, 95, 18, 32, 165, 255, 112, 95, 218, 79, 38, 66, 15, 168, 89, 176, 189, 174, 15, 225, 172, 165, 132, 159, 23, 89]
				)
			]
		);

	});
}
