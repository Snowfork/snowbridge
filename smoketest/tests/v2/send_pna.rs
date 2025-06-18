use alloy::primitives::{Address, Bytes, U256};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::i_gateway_v2 as i_gateway,
	helper::{
		build_native_asset, get_token_address, initial_clients, print_event_log_for_unit_tests,
	},
	parachains::assethub::api::{
		balances::events::Minted,
		runtime_types::{
			staging_xcm::v5::{
				asset::{AssetFilter::Wild, WildAsset::AllCounted},
				junction::Junction::AccountId32,
				junctions::Junctions,
				location::Location,
				Instruction::DepositAsset,
				Xcm,
			},
			xcm::VersionedXcm,
		},
	},
};
use subxt::{ext::codec::Encode, utils::AccountId32 as AccountId32Substrate};

#[tokio::test]
async fn send_pna() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let token_address = get_token_address(ethereum_client, ERC20_DOT_TOKEN_ID).await.unwrap();

	let execution_fee = 2_000_000_000_000u128;
	let relayer_fee = 2_000_000_000u128;
	let fee = 9_000_000_000_000u128;

	let amount: u128 = 10_000_000_000;

	let pna_asset = build_native_asset(token_address, amount);
	let beneficiary = Location {
		parents: 0,
		interior: Junctions::X1([AccountId32 { network: None, id: (*SUBSTRATE_RECEIVER).into() }]),
	};

	let message = VersionedXcm::V5(Xcm(vec![DepositAsset {
		assets: Wild(AllCounted(2)),
		beneficiary: beneficiary.clone(),
	}]));
	let encoded_xcm = message.encode();

	let xcm = Bytes::from(encoded_xcm);
	let claimer = Bytes::from(beneficiary.clone().encode());
	let assets = vec![pna_asset];

	let receipt = gateway
		.v2_sendMessage(xcm, assets, claimer, execution_fee, relayer_fee)
		.value(U256::from(fee))
		.gas_price(GAS_PRICE)
		.send()
		.await
		.unwrap()
		.get_receipt()
		.await
		.expect("get receipt");

	println!(
		"receipt transaction hash: {:#?}, transaction block: {:#?}",
		hex::encode(receipt.transaction_hash),
		receipt.block_number
	);

	// Log for OutboundMessageAccepted
	let outbound_message_accepted_log = receipt.logs().last().unwrap().as_ref();

	// print log for unit tests
	print_event_log_for_unit_tests(outbound_message_accepted_log);

	assert_eq!(receipt.status(), true);

	let wait_for_blocks = (*WAIT_PERIOD) as usize;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_owner: AccountId32Substrate = (*SUBSTRATE_RECEIVER).into();

	let mut event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for minted event.", block.number());

		let events = block.events().await.unwrap();
		for event in events.find::<Minted>() {
			let minted = event.unwrap();
			assert_eq!(minted.who, expected_owner);
			assert_eq!(minted.amount, amount);
			event_found = true;
		}
		if event_found {
			break;
		}
	}
	assert!(event_found);
}
