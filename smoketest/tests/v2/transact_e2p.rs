use ethers::{core::types::Address, types::Bytes};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::i_gateway_v2 as i_gateway,
	helper::{initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::{
		runtime_types::{
			staging_xcm::v5::{
				Instruction::{ExpectTransactStatus, Transact},
				Xcm,
			},
			xcm::{
				double_encoded::DoubleEncoded,
				v3::{MaybeErrorCode, OriginKind},
				VersionedXcm,
			},
		},
		system::events::Remarked,
	},
};
use sp_crypto_hashing::blake2_256;
use subxt::{ext::codec::Encode, tx::Payload};

#[tokio::test]
async fn transact_e2p() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let execution_fee = 2_000_000_000u128;
	let relayer_fee = 2_000_000_000u128;
	let fee = 9_000_000_000u128;

	let remark_message = b"Hey there";
	let mut encoded = Vec::new();
	let system_api = snowbridge_smoketest::parachains::assethub::api::system::calls::TransactionApi;
	system_api
		.remark_with_event(remark_message.into())
		.encode_call_data_to(&assethub.metadata(), &mut encoded)
		.expect("encoded call");

	let message = VersionedXcm::V5(Xcm(vec![
		Transact {
			origin_kind: OriginKind::SovereignAccount,
			fallback_max_weight: None,
			call: DoubleEncoded { encoded },
		},
		ExpectTransactStatus(MaybeErrorCode::Success),
	]));
	let encoded_xcm = message.encode();

	let xcm = Bytes::from(encoded_xcm);
	let claimer = Bytes::from(vec![]);
	let assets = vec![];

	let receipt = gateway
		.v_2_send_message(xcm, assets, claimer, execution_fee, relayer_fee)
		.value(fee)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();

	println!(
		"receipt transaction hash: {:#?}, transaction block: {:#?}",
		hex::encode(receipt.transaction_hash),
		receipt.block_number
	);

	let outbound_message_accepted_log = receipt.logs.last().unwrap();

	print_event_log_for_unit_tests(outbound_message_accepted_log);

	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let wait_for_blocks = (*WAIT_PERIOD) as usize;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let mut remark_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for event in events.find::<Remarked>() {
			let event = event.unwrap();
			let remark_hash = blake2_256(remark_message);
			assert_eq!(event.hash, remark_hash.into());
			remark_event_found = true;
		}
		if remark_event_found {
			break
		}
	}
	assert!(remark_event_found);
}
