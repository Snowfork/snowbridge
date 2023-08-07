use ethers::{
	core::types::{Address, U256},
	middleware::SignerMiddleware,
	providers::{Http, Provider},
	signers::{LocalWallet, Signer},
	utils::parse_units,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	contracts::{i_gateway, weth9},
	parachains::assethub::api::{
		foreign_assets::events::Issued,
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
use sp_core::Encode;
use std::{sync::Arc, time::Duration};
use subxt::{utils::AccountId32, OnlineClient, PolkadotConfig};

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify the order in
// contracts are deployed in DeployScript.sol.
const ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:13144";
const ETHEREUM_API: &str = "http://localhost:8545";
const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
const WETH_CONTRACT: [u8; 20] = hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");
const GATEWAY_PROXY_CONTRACT: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");

// SS58: DE14BzQ1bDXWPKeLoAqdLAm1GpyAWaWF1knF74cEZeomTBM
const FERDIE: [u8; 32] = hex!("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c");

// TODO: test sendNativeToken
#[tokio::test]
async fn send_token() {
	let provider = Provider::<Http>::try_from(ETHEREUM_API)
		.unwrap()
		.interval(Duration::from_millis(10u64));

	let wallet: LocalWallet = ETHEREUM_KEY.parse::<LocalWallet>().unwrap().with_chain_id(15u64);

	let client = SignerMiddleware::new(provider.clone(), wallet.clone());
	let client = Arc::new(client);

	let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
	let gateway = i_gateway::IGateway::new(gateway_addr, client.clone());

	let weth_addr: Address = WETH_CONTRACT.into();
	let weth = weth9::WETH9::new(weth_addr, client.clone());

	let assethub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url(ASSET_HUB_WS_URL).await.unwrap();

	// Mint WETH tokens
	let value = parse_units("1", "ether").unwrap();
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

	// Lock tokens into vault
	let amount: u128 = U256::from(value).low_u128();
	let receipt = gateway
		.send_token(weth.address(), 1000.into(), FERDIE, amount)
		.value(1000)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();

	println!("receipt: {:#?}", receipt);

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
	let expected_owner: AccountId32 = FERDIE.into();

	let mut issued_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<Issued>() {
			println!("Created event found in assethub block {}.", block.number());
			let issued = issued.unwrap();
			assert_eq!(issued.asset_id.encode(), expected_asset_id.encode());
			assert_eq!(issued.owner, expected_owner);
			assert_eq!(issued.amount, amount);
			issued_event_found = true;
		}
		if issued_event_found {
			break;
		}
	}
	assert!(issued_event_found)
}
