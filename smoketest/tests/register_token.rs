use codec::Encode;
use ethers::{
	core::types::Address,
	middleware::SignerMiddleware,
	providers::{Http, Provider},
	signers::{LocalWallet, Signer},
	utils::{parse_units, rlp::Encodable},
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	contracts::{i_gateway, weth9},
	parachains::assethub::api::{
		foreign_assets::events::Created,
		runtime_types::xcm::v3::{
			junction::{
				Junction::{AccountKey20, GlobalConsensus},
				NetworkId,
			},
			junctions::Junctions::X3,
			multilocation::MultiLocation,
		},
	},
};
use sp_core::blake2_256;
use std::{sync::Arc, time::Duration};
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify
// the order in contracts are deployed in DeployScript.sol.
const ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:12144";
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const GATEWAY_PROXY_CONTRACT: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");
const WETH_CONTRACT: [u8; 20] = hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");
const GATEWAY_PROXY_SOVEREIGN: [u8; 32] =
	hex!("c9794dd8013efb2ad83f668845c62b373c16ad33971745731408058e4d0c6ff5");
const ASSET_HUB_AGENT_ID: [u8; 32] =
	hex!("72456f48efed08af20e5b317abf8648ac66e86bb90a411d9b0b713f7364b75b4");

#[tokio::test]
async fn register_token() {
	let provider = Provider::<Http>::try_from(ETHEREUM_API)
		.unwrap()
		.interval(Duration::from_millis(10u64));
	let wallet: LocalWallet = ETHEREUM_KEY.parse::<LocalWallet>().unwrap().with_chain_id(15u64);
	let client = SignerMiddleware::new(provider, wallet.clone());
	let client = Arc::new(client);

	let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
	let gateway = i_gateway::IGateway::new(gateway_addr, client.clone());

	let weth_addr: Address = WETH_CONTRACT.into();
	let weth = weth9::WETH9::new(weth_addr, client.clone());

	let assethub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

	let fee = parse_units(2, "ether").unwrap();

	let receipt = gateway
		.register_token(weth.address())
		.value(fee)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();

	println!("receipt transaction hash: {:#?}", hex::encode(receipt.transaction_hash));

	// Log for OutboundMessageAccepted
	let outbound_message_accepted_log = receipt.logs.last().unwrap();
	// RLP-encode log and print it
	println!("receipt log: {:#?}", hex::encode(outbound_message_accepted_log.rlp_bytes()));

	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let wait_for_blocks = 50;
	let mut blocks = assethub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_asset_id: MultiLocation = MultiLocation {
		parents: 2,
		interior: X3(
			GlobalConsensus(NetworkId::Ethereum { chain_id: 15 }),
			AccountKey20 { network: None, key: GATEWAY_PROXY_CONTRACT.into() },
			AccountKey20 { network: None, key: WETH_CONTRACT.into() },
		),
	};
	let expected_creator: AccountId32 = GATEWAY_PROXY_SOVEREIGN.into();
	let agent_address = gateway.agent_of(ASSET_HUB_AGENT_ID).await.unwrap();
	let expected_owner = AccountId32::from(
		(b"ethereum", 15u64, agent_address.as_fixed_bytes()).using_encoded(blake2_256),
	);
	println!("sender address: {}", expected_owner.to_string());

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
