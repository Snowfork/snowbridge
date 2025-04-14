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
	parachains::penpal::api::{
		foreign_assets::events::{Issued as PenpalIssued, Issued as AssetHubIssued},
		runtime_types::{
			staging_xcm::v5::{
				asset::{
					Asset,
					AssetFilter::{Definite, Wild},
					AssetId, Assets, Fungibility,
					WildAsset::AllCounted,
				},
				junction::Junction::{AccountId32, Parachain},
				junctions::Junctions::X1,
				location::Location,
				Instruction::{
					DepositAsset, DepositReserveAsset, PayFees, RefundSurplus, SetTopic,
				},
				Xcm,
			},
			xcm::VersionedXcm,
		},
	},
	penpal_helper::{
		create_asset_pool, dot_location, ensure_penpal_asset_exists, eth_location,
		set_reserve_asset_storage, weth_location, PenpalConfig,
	},
};
use subxt::{
	config::substrate::H256, ext::codec::Encode, utils::AccountId32 as AccountId32Substrate,
	OnlineClient,
};

#[tokio::test]
async fn send_ena_to_penpal() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub_client = *(test_clients.asset_hub_client.clone());
	let penpal_client: OnlineClient<PenpalConfig> = OnlineClient::from_url(PENPAL_WS_URL)
		.await
		.expect("can not connect to penpal parachain");

	set_reserve_asset_storage(&mut penpal_client.clone()).await;
	ensure_penpal_asset_exists(&mut penpal_client.clone(), weth_location()).await;
	ensure_penpal_asset_exists(&mut penpal_client.clone(), dot_location()).await;
	ensure_penpal_asset_exists(&mut penpal_client.clone(), eth_location()).await;

	create_asset_pool(&Box::new(penpal_client.clone()), &Box::new(assethub_client.clone())).await;

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

	let execution_fee = 1_500_000_000_000u128;
	let relayer_fee = 1_500_000_000_000u128;
	let fee = 5_500_000_000_000u128;

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let amount: u128 = U256::from(value).low_u128();
	let weth_asset = build_native_asset(weth.address(), amount);
	// To pay fees on Penpal.
	let eth_fee_penpal: Asset =
		Asset { id: AssetId(eth_location()), fun: Fungibility::Fungible(3_000_000_000_000u128) };
	let eth_fee_penpal2: Asset =
		Asset { id: AssetId(eth_location()), fun: Fungibility::Fungible(2_000_000_000_000u128) };
	let token_asset: Asset =
		Asset { id: AssetId(weth_location()), fun: Fungibility::Fungible(amount) };

	let message = VersionedXcm::V5(Xcm(vec![DepositReserveAsset {
		// Send the token plus some eth for execution fees
		assets: Definite(Assets(vec![eth_fee_penpal, token_asset])),
		// Penpal
		dest: Location { parents: 1, interior: X1([Parachain(PENPAL_PARA_ID)]) },
		xcm: Xcm(vec![
			// Pay fees on Penpal.
			PayFees { asset: eth_fee_penpal2 },
			// Deposit assets to beneficiary.
			DepositAsset { assets: Wild(AllCounted(2)), beneficiary: beneficiary() },
			RefundSurplus,
			DepositAsset { assets: Wild(AllCounted(2)), beneficiary: beneficiary() },
			SetTopic(H256::random().into()),
		]),
	}]));

	let xcm = Bytes::from(message.encode());
	let claimer = Bytes::from(beneficiary().encode());
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
	let mut blocks = assethub_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let mut issued_event_found = false;
	let mut issued_fee_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<AssetHubIssued>() {
			let issued = issued.unwrap();
			if issued.asset_id.encode() == weth_location().encode() {
				println!("Issued Weth event found in assethub block {}.", block.number());
				assert_eq!(issued.asset_id.encode(), weth_location().encode());
				issued_event_found = true;
			} else if issued.asset_id.encode() == eth_location().encode() {
				println!("Issued Eth event found in assethub block {}.", block.number());
				assert_eq!(issued.asset_id.encode(), eth_location().encode());
				issued_fee_event_found = true;
			}
		}
		if issued_event_found && issued_fee_event_found {
			break
		}
	}
	assert!(issued_event_found);
	assert!(issued_fee_event_found);

	let expected_owner: AccountId32Substrate = (*SUBSTRATE_RECEIVER).into();

	let mut penpal_blocks = penpal_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let mut issued_event_found = false;
	let mut issued_fee_event_found = false;
	while let Some(Ok(block)) = penpal_blocks.next().await {
		println!("Polling penpal block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<PenpalIssued>() {
			let issued = issued.unwrap();
			// ETH fee deposited
			if issued.asset_id.encode() == eth_location().encode() {
				println!("Issued Eth event found in penpal block {}.", block.number());
				assert_eq!(issued.owner, expected_owner);
				issued_fee_event_found = true
			}
			// Weth deposited
			if issued.asset_id.encode() == weth_location().encode() {
				println!("Issued Weth event found in penpal block {}.", block.number());
				assert_eq!(issued.owner, expected_owner);
				assert_eq!(issued.amount, amount);
				issued_event_found = true;
			}
		}
		if issued_event_found && issued_fee_event_found {
			break
		}
	}
	assert!(issued_event_found);
	assert!(issued_fee_event_found);
}

fn beneficiary() -> Location {
	Location {
		parents: 0,
		interior: X1([AccountId32 { network: None, id: (*SUBSTRATE_RECEIVER).into() }]),
	}
}
