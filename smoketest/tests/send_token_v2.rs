use ethers::{
	core::types::Address,
	types::{Bytes, U256},
	utils::parse_units,
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway_v2 as i_gateway, weth9},
	helper::{initial_clients, print_event_log_for_unit_tests},
	helper_v2::build_native_asset,
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
async fn send_token_v2() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub = *(test_clients.asset_hub_client.clone());

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	// Mint WETH tokens
	let value = parse_units("0.01", "ether").unwrap();
	let receipt = weth.deposit().value(value).send().await.unwrap().await.unwrap().unwrap();
	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	// Approve token spend
	weth.approve(gateway_addr, value.into())
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let execution_fee = 2_000_000_000u128;
	let relayer_fee = 2_000_000_000u128;
	let fee = 9_000_000_000u128;

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let amount: u128 = U256::from(value).low_u128();
	let weth_asset = build_native_asset(weth.address(), amount);
	let beneficiary = Location {
		parents: 0,
		interior: Junctions::X1([AccountId32 { network: None, id: (*SUBSTRATE_RECEIVER).into() }]),
	};

	let weth_location = Location {
		parents: 2,
		interior: Junctions::X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: weth.address().into() },
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

	// Log for OutboundMessageAccepted
	let outbound_message_accepted_log = receipt.logs.last().unwrap();

	// print log for unit tests
	print_event_log_for_unit_tests(outbound_message_accepted_log);

	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

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
	let expected_eth_id: Location = Location {
		parents: 2,
		interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	};
	let expected_owner: AccountId32Substrate = (*SUBSTRATE_RECEIVER).into();

	let mut issued_weth_event_found = false;
	let mut issued_eth_event_found = false;
	let mut event_count = 0;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<Issued>() {
			let issued = issued.unwrap();
			event_count = event_count + 1;
			if event_count == 1 {
				// Issued weth token
				assert_eq!(issued.asset_id.encode(), expected_weth_id.encode());
				assert_eq!(issued.owner, expected_owner);
				assert_eq!(issued.amount, amount);
				issued_weth_event_found = true;
			} else if event_count == 2 {
				// Refunded eth fees
				assert_eq!(issued.asset_id.encode(), expected_eth_id.encode());
				issued_eth_event_found = true;
			}
		}
		if event_count >= 2 {
			break
		}
	}
	assert!(issued_weth_event_found);
	assert!(issued_eth_event_found);
}
