use alloy::primitives::{utils::parse_units, Address, Bytes, U256};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway_v2 as i_gateway, weth9},
	helper::{build_native_asset, initial_clients, print_event_log_for_unit_tests},
	parachains::assethub::api::{
		foreign_assets::events::Issued,
		runtime_types::{
			staging_xcm::v5::{
				asset::{AssetFilter::Wild, AssetId, WildAsset::AllOf, WildFungibility},
				junction::{
					Junction::{AccountId32, AccountKey20, GlobalConsensus},
					NetworkId,
				},
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
async fn send_ena_to_ah() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	// Register WETH token  (Weth is already pre-registered in the Westend genesis config,
	// but it is not registered on Ethereum).
	let register_receipt = gateway
		.v2_registerToken(*weth.address(), 0, 1_500_000_000_000u128, 1_500_000_000_000u128)
		.value(U256::from(13_000_000_000_000u128))
		.gas_price(GAS_PRICE)
		.send()
		.await
		.unwrap()
		.get_receipt()
		.await
		.expect("register WETH");
	assert_eq!(register_receipt.status(), true);

	// Mint WETH tokens
	let value = parse_units("0.01", "ether").unwrap().get_absolute();
	let mut receipt = weth
		.deposit()
		.value(value)
		.gas_price(GAS_PRICE)
		.send()
		.await
		.unwrap()
		.get_receipt()
		.await
		.expect("get receipt");
	assert_eq!(receipt.status(), true);

	// Approve token spend
	receipt = weth
		.approve(gateway_addr, value.into())
		.gas_price(GAS_PRICE)
		.send()
		.await
		.unwrap()
		.get_receipt()
		.await
		.expect("get receipt");

	assert_eq!(receipt.status(), true);

	let execution_fee = 2_000_000_000_000u128;
	let relayer_fee = 200_000_000_000u128;
	let fee = 9_000_000_000_000u128;

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let amount: u128 = value.to::<u128>();
	let weth_asset = build_native_asset(*weth.address(), amount);
	let beneficiary = Location {
		parents: 0,
		interior: Junctions::X1([AccountId32 { network: None, id: (*SUBSTRATE_RECEIVER).into() }]),
	};

	let weth_location = Location {
		parents: 2,
		interior: Junctions::X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*weth.address()).into() },
		]),
	};
	let message = VersionedXcm::V5(Xcm(vec![DepositAsset {
		assets: Wild(AllOf { id: AssetId(weth_location), fun: WildFungibility::Fungible }),
		beneficiary,
	}]));
	let encoded_xcm = message.encode();

	let xcm = Bytes::from(encoded_xcm);
	let claimer = Bytes::from(vec![]);
	let assets = vec![weth_asset];

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

	let expected_weth_id: Location = Location {
		parents: 2,
		interior: Junctions::X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	};
	let expected_owner: AccountId32Substrate = (*SUBSTRATE_RECEIVER).into();

	let mut issued_weth_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<Issued>() {
			let issued = issued.unwrap();
			if issued.asset_id.encode() != expected_weth_id.encode() { // skip unrelated events
				continue
			}
			// Issued weth token
			assert_eq!(issued.asset_id.encode(), expected_weth_id.encode());
			assert_eq!(issued.owner, expected_owner);
			assert_eq!(issued.amount, amount);
			issued_weth_event_found = true;
		}
		if issued_weth_event_found {
			break;
		}
	}
	assert!(issued_weth_event_found);
}
