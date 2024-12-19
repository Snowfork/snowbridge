use ethers::{
	core::types::{Address, U256},
	utils::parse_units,
};
use futures::StreamExt;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway, weth9},
	helper::initial_clients,
	parachains::{
		assethub::api::{
			foreign_assets::events::Issued as AssetHubIssued,
			runtime_types::staging_xcm::v4::{
				junction::{
					Junction::{AccountKey20, GlobalConsensus},
					NetworkId,
				},
				junctions::Junctions::{Here, X2},
				location::Location,
			},
		},
		penpal::{self, api::foreign_assets::events::Issued as PenpalIssued},
	},
	penpal_helper::PenpalConfig,
};
use subxt::{
	ext::codec::Encode,
	tx::PairSigner,
	utils::{AccountId32, MultiAddress},
	OnlineClient,
};
use sp_crypto_hashing::twox_128;
use penpal::{api::runtime_types as penpalTypes};
use penpalTypes::{
	penpal_runtime::RuntimeCall as PenpalRuntimeCall,
};

#[tokio::test]
async fn send_token_to_penpal() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = *(test_clients.ethereum_signed_client.clone());
	let assethub_client = *(test_clients.asset_hub_client.clone());
	let penpal_client: OnlineClient<PenpalConfig> = OnlineClient::from_url(PENPAL_WS_URL)
		.await
		.expect("can not connect to penpal parachain");

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());

	let weth_addr: Address = (*WETH_CONTRACT).into();
	let weth = weth9::WETH9::new(weth_addr, ethereum_client.clone());

	// Mint WETH tokens
	let value = parse_units("1", "ether").unwrap();
	let receipt = weth.deposit().value(value).send().await.unwrap().await.unwrap().unwrap();
	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	set_reserve_asset_storage(&mut penpal_client.clone()).await;
	ensure_penpal_asset_exists(&mut penpal_client.clone()).await;
	ensure_dot_asset_exists(&mut penpal_client.clone()).await;

	// Approve token spend
	weth.approve(gateway_addr, value.into())
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();
	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let destination_fee = 4_000_000_000;
	let fee = gateway
		.quote_send_token_fee(weth.address(), PENPAL_PARA_ID, destination_fee)
		.call()
		.await
		.unwrap();

	// Lock tokens into vault
	let amount: u128 = U256::from(value).low_u128();
	let receipt = gateway
		.send_token(
			weth.address(),
			PENPAL_PARA_ID,
			i_gateway::MultiAddress { kind: 1, data: (*FERDIE_PUBLIC).into() },
			4_000_000_000,
			amount,
		)
		.value(fee)
		.send()
		.await
		.unwrap()
		.await
		.unwrap()
		.unwrap();

	println!("receipt: {:#?}", receipt);

	assert_eq!(receipt.status.unwrap().as_u64(), 1u64);

	let wait_for_blocks = 100;
	let mut assethub_blocks = assethub_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let expected_dot_id = Location {
		parents: 1,
		interior: Here,
	};
	let expected_asset_id = Location {
		parents: 2,
		interior: X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	};
	let assethub_expected_owner: AccountId32 = PENPAL_SOVEREIGN.into();

	let mut issued_event_found = false;
	while let Some(Ok(block)) = assethub_blocks.next().await {
		println!("Polling assethub block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<AssetHubIssued>() {
			println!("Created event found in assethub block {}.", block.number());
			let issued = issued.unwrap();
			assert_eq!(issued.asset_id.encode(), expected_asset_id.encode());
			assert_eq!(issued.owner, assethub_expected_owner);
			assert_eq!(issued.amount, amount);
			issued_event_found = true;
		}
		if issued_event_found {
			break
		}
	}
	assert!(issued_event_found);

	let mut penpal_blocks = penpal_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let penpal_expected_owner: AccountId32 = (*FERDIE_PUBLIC).into();

	issued_event_found = false;
	let mut issued_fee_event_found = false;
	while let Some(Ok(block)) = penpal_blocks.next().await {
		println!("Polling penpal block {} for issued event.", block.number());

		let events = block.events().await.unwrap();
		for issued in events.find::<PenpalIssued>() {
			let issued = issued.unwrap();
			// DOT fee deposited
			if issued.asset_id.encode() == expected_dot_id.encode() {
				println!("Issued DOT event found in penpal block {}.", block.number());
				assert_eq!(issued.owner, penpal_expected_owner);
				issued_fee_event_found = true
			}
			// Weth deposited
			if issued.asset_id.encode() == expected_asset_id.encode() {
				println!("Issued Weth event found in penpal block {}.", block.number());
				assert_eq!(issued.owner, penpal_expected_owner);
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

async fn ensure_penpal_asset_exists(penpal_client: &mut OnlineClient<PenpalConfig>) {
	use penpal::api::runtime_types::staging_xcm::v4::{
		junction::{
			Junction::{AccountKey20, GlobalConsensus},
			NetworkId,
		},
		junctions::Junctions::X2,
		location::Location,
	};
	let penpal_asset_id = Location {
		parents: 2,
		interior: X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	};

	let penpal_asset_address = penpal::api::storage().foreign_assets().asset(&penpal_asset_id);
	let result = penpal_client
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&penpal_asset_address)
		.await
		.unwrap();

	if result.is_some() {
		println!("WETH asset exists on penpal.");
		return
	}

	println!("creating WETH on penpal.");
	let admin = MultiAddress::Id(ASSET_HUB_SOVEREIGN.into());
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*FERDIE).clone());

	let create_asset_call = penpal::api::tx().foreign_assets().create(penpal_asset_id, admin, 1);
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&create_asset_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("asset created");
}

async fn ensure_dot_asset_exists(penpal_client: &mut OnlineClient<PenpalConfig>) {
	use penpal::api::runtime_types::staging_xcm::v4::{
		junctions::Junctions::Here,
		location::Location,
	};
	let dot_asset_id = Location {
		parents: 1,
		interior: Here,
	};

	let dot_asset_address = penpal::api::storage().foreign_assets().asset(&dot_asset_id);
	let result = penpal_client
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&dot_asset_address)
		.await
		.unwrap();

	if result.is_some() {
		println!("DOT asset exists on penpal.");
		return
	}

	println!("creating DOT on penpal.");
	let admin = MultiAddress::Id(ASSET_HUB_SOVEREIGN.into());
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*FERDIE).clone());

	let create_asset_call = penpal::api::tx().foreign_assets().create(dot_asset_id, admin, 1);
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&create_asset_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("asset created");
}
async fn set_reserve_asset_storage(penpal_client: &mut OnlineClient<PenpalConfig>) {
	use penpal::api::runtime_types::staging_xcm::v4::{
		junction::{
			Junction::GlobalConsensus,
			NetworkId,
		},
		junctions::Junctions::X1,
		location::Location,
	};
	let storage_key: Vec<u8> = twox_128(b":CustomizableAssetFromSystemAssetHub:").to_vec();
	let reserve_location: Vec<u8> = Location {
		parents: 2,
		interior: X1([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
		]),
	}.encode();

	println!("setting CustomizableAssetFromSystemAssetHub storage on penpal.");
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*ALICE).clone());

	let items = vec![(
		storage_key,
		reserve_location,
	)];

	let sudo_call = penpal::api::sudo::calls::TransactionApi::sudo(
		&penpal::api::sudo::calls::TransactionApi,
		PenpalRuntimeCall::System(crate::penpalTypes::frame_system::pallet::Call::set_storage {
			items
		}),
	);
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("reserve location set");
}
