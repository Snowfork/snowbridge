use hex_literal::hex;
use snowbridge_beacon_primitives::CompactExecutionHeader;
use snowbridge_core::inbound::{Log, Message, Proof};
use sp_std::vec;

pub struct InboundQueueTest {
	pub execution_header: CompactExecutionHeader,
	pub message: Message,
}

pub fn make_create_message() -> InboundQueueTest {
	InboundQueueTest{
        execution_header: CompactExecutionHeader{
            parent_hash: hex!("9e2078694f20148b48e938a5b35a4cca79e19a05b7f27c7b3daae11a2ab57524").into(),
            block_number: 55,
            state_root: hex!("74865f49fe887e1b9df502282b1e99ccf563861a0ed58e9e541d966207d11f3f").into(),
            receipts_root: hex!("0115ab735d37c5e4cdb0374d8bb547c6dd6ccaa996d996d1eabc5399a719219e").into(),
        },
        message: Message {
            event_log: 	Log {
                address: hex!("eda338e4dc46038493b885327842fd3e301cab39").into(),
                topics: vec![
                    hex!("5066fbba677e15936860e04088ca4cad3acd4c19706962196a5346f1457f7169").into(),
                    hex!("00000000000000000000000000000000000000000000000000000000000003e8").into(),
                    hex!("afad3c9777134532ae230b4fad334eef2e0dacbb965920412a7eaa59b07d640f").into(),
                ],
                data: hex!("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000001e000f000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000").into(),
            },
            proof: Proof {
                block_hash: hex!("5f465744c166e9d10dc0031942a59ff82b640053253da517a1b576afdadb0363").into(),
                tx_index: 0,
                data: (vec![
                    hex!("0115ab735d37c5e4cdb0374d8bb547c6dd6ccaa996d996d1eabc5399a719219e").to_vec(),
                    hex!("caf5ee6beba6a6db5e2a0714a98f65ac4365c4a24e56ce033f19c7f8a2abb06a").to_vec(),
                ], vec![
                    hex!("5e2a0714a98f65ac4365c4a24e56ce033f19c7f8a2abb06a8080808080808080").to_vec(),
                    hex!("000f000000000000000087d1f7fdfee7f651fabc8bfcb6e086c278b77a7d0000").to_vec(),
                ]),
            },
        },
    }
}
