use codec::Encode;
use ethers::core::types::Address;
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway_v2 as i_gateway, weth9},
	helper::{initial_clients, print_event_log_for_unit_tests, governance_assethub_call_from_relay_chain, fund_account_on_relaychain},
	parachains::assethub::api::{
		foreign_assets::events::Created,
		runtime_types::{
			staging_xcm::v5::{
				junction::{Junction::GlobalConsensus, NetworkId, Junction::{AccountKey20}},
				junctions::{Junctions::{X1, X2, Here}},
				location::Location,
			},
		},
	},
};
use snowbridge_smoketest::helper::AssetHubConfig;
use subxt::{
	tx::PairSigner,
	utils::{AccountId32, MultiAddress},
	OnlineClient,
};
use snowbridge_smoketest::parachains::assethub::api::asset_conversion::events::{PoolCreated, LiquidityAdded};
use subxt::tx::Payload;

#[tokio::test]
async fn register_token_v2() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let mut assethub = *(test_clients.asset_hub_client.clone());

	//create_asset_pool(&mut assethub).await;

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	let receipt = gateway
		.v_2_register_token(weth.address(), 0, 1_500_000_000_000u128, 1_500_000_000_000u128)
		.value(13_000_000_000_000u128)
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

	let expected_asset_id: Location = Location {
		parents: 2,
		interior: X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
	]),
	};
	let expected_creator: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();
	let expected_owner: AccountId32 = SNOWBRIDGE_SOVEREIGN.into();

	let mut created_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for created event.", block.number());

		let events = block.events().await.unwrap();
		for created in events.find::<Created>() {
			println!("Created event found in assethub block {}.", block.number());
			let created = created.unwrap();
			assert_eq!(created.asset_id.encode(), expected_asset_id.encode());
			assert_eq!(created.creator, expected_creator);
			assert_eq!(created.owner, expected_owner);
			created_event_found = true;
		}
		if created_event_found {
			break
		}
	}
	assert!(created_event_found)
}

async fn create_asset_pool(asset_hub_client: &mut OnlineClient<AssetHubConfig>) {
	let foreign_assets_api = snowbridge_smoketest::parachains::assethub::api::foreign_assets::calls::TransactionApi;

	// Mint eth to sovereign account
	let admin = MultiAddress::Id(SNOWBRIDGE_SOVEREIGN.into());
	let mut encoded = Vec::new();
	foreign_assets_api
		.mint(eth_location(), admin.clone(), 3_500_000_000_000)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded)
		.expect("encoded call");

	//governance_assethub_call_from_relay_chain(admin.clone(), encoded)
	//	.await
	//	.expect("fund snowbridge sovereign with eth for pool");

	let ferdie_account: AccountId32 = (*FERDIE_PUBLIC).into();
	let mut encoded2 = Vec::new();
	foreign_assets_api
		.transfer(eth_location(), MultiAddress::Id(ferdie_account.clone()), 3_000_000_000_000)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded2)
		.expect("encoded call");

	//governance_assethub_call_from_relay_chain(admin.clone(), encoded2)
	//	.await
	//	.expect("transfer eth to ferdie");

	let create_pool_call = snowbridge_smoketest::parachains::assethub::api::tx().asset_conversion().create_pool(dot_location(), eth_location());
	let signer: PairSigner<AssetHubConfig, _> = PairSigner::new((*FERDIE).clone());
	//asset_hub_client
	//	.tx()
	//	.sign_and_submit_then_watch_default(&create_pool_call, &signer)
	//	.await
	//	.unwrap()
	//	.wait_for_finalized_success()
	//	.await
	//	.expect("pool created");
//
	//let wait_for_blocks = (*WAIT_PERIOD) as usize;
	//let mut blocks = asset_hub_client
	//	.blocks()
	//	.subscribe_finalized()
	//	.await
	//	.expect("block subscription")
	//	.take(wait_for_blocks);
//
	//let mut pool_event_found = false;
	//while let Some(Ok(block)) = blocks.next().await {
	//	println!("Polling assethub block {} for pool created event.", block.number());
//
	//	let events = block.events().await.unwrap();
	//	for _pool_created in events.find::<PoolCreated>() {
	//		println!("Pool created event found in assethub block {}.", block.number());
	//		pool_event_found = true;
	//	}
	//	if pool_event_found {
	//		break
	//	}
	//}
	//assert!(pool_event_found);



	// add liquidity
	let create_liquidity = snowbridge_smoketest::parachains::assethub::api::tx().asset_conversion().add_liquidity(dot_location(), eth_location(), 1_000_000_000_000, 2_000_000_000_000, 1, 1, ferdie_account);
	let signer: PairSigner<AssetHubConfig, _> = PairSigner::new((*FERDIE).clone());
	asset_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&create_liquidity, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("liquidity added");

	let wait_for_blocks_liquidity = (*WAIT_PERIOD) as usize;
	let mut blocks = asset_hub_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks_liquidity);

	let mut liquidity_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for liquidity added event.", block.number());

		let events = block.events().await.unwrap();
		for _liquidity_event_found in events.find::<LiquidityAdded>() {
			println!("Liquidity added event found in assethub block {}.", block.number());
			liquidity_event_found = true;
		}
		if liquidity_event_found {
			break
		}
	}
	assert!(liquidity_event_found)
}

fn eth_location() -> Location {
	Location {
		parents: 2,
		interior: X1([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
		]),
	}
}

fn dot_location() -> Location {
	Location {
		parents: 1,
		interior: Here
	}
}
