use crate::{mock::*};

use crate::{Message, MessageQueues};

use sp_runtime::DigestItem;
use sp_core::H160;

use frame_support::{
	traits::{OnInitialize}
};

use frame_support::storage::StorageMap;

use artemis_core::{ChannelId, MessageCommitment};

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
		CommitmentsModule::add(ChannelId::Basic, CONTRACT_A, 0, &vec![0, 1, 2]).unwrap();
		CommitmentsModule::add(ChannelId::Basic, CONTRACT_B, 1, &vec![3, 4, 5]).unwrap();

		let messages = vec![
			Message {
				target: CONTRACT_A,
				nonce: 0,
				payload: vec![0, 1, 2],
			},
			Message {
				target: CONTRACT_B,
				nonce: 1,
				payload: vec![3, 4, 5],
			},
		];

		assert_eq!(
			MessageQueues::get(ChannelId::Basic), messages);

		// Run to block 5 where a commitment will be generated
		run_to_block(5);

		assert_eq!(
			MessageQueues::contains_key(ChannelId::Basic), false
		);
		assert_eq!(
			System::digest().logs(),
			vec![
				DigestItem::Other(vec![0, 0, 48, 89, 246, 187, 20, 156, 87, 142, 138, 90, 46, 234, 197, 120, 204, 50, 208, 209, 63, 125, 48, 204, 124, 195, 132, 234, 48, 140, 24, 59, 6, 244])
			]
		);

	});
}
