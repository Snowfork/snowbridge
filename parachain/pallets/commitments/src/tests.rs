use crate::mock::{new_test_ext, Commitments, System};

use crate::{BasicMessageQueue, Message};

use sp_core::H160;
use sp_runtime::DigestItem;

use frame_support::traits::OnInitialize;

use frame_support::storage::StorageMap;

use artemis_core::{BasicMessageCommitment, ChannelId, IncentivizedMessageCommitment};

fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
		Commitments::on_initialize(System::block_number());
	}
}

const CONTRACT_A: H160 = H160::repeat_byte(1);
const CONTRACT_B: H160 = H160::repeat_byte(2);

#[test]
fn test_add_message() {
	new_test_ext().execute_with(|| {
		Commitments::add_basic(ChannelId::Basic, CONTRACT_A, 0, &vec![0, 1, 2]).unwrap();
		Commitments::add_basic(ChannelId::Basic, CONTRACT_B, 1, &vec![3, 4, 5]).unwrap();

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

		assert_eq!(MessageQueues::get(ChannelId::Basic), messages);

		// Run to block 5 where a commitment will be generated
		run_to_block(5);

		assert_eq!(MessageQueues::contains_key(ChannelId::Basic), false);
		assert_eq!(
			System::digest().logs(),
			vec![DigestItem::Other(vec![
				0, 0, 75, 224, 75, 115, 209, 7, 157, 71, 172, 222, 139, 122, 150, 76, 83, 255, 213,
				213, 15, 233, 253, 193, 12, 4, 71, 27, 94, 86, 44, 150, 225, 60
			])]
		);
	});
}
