use crate::{
	constants::*,
	helper::{snowbridge_assethub_call_from_relay_chain, AssetHubConfig},
	parachains::{
		assethub,
		assethub::api::runtime_types::staging_xcm::v5::{
			junction::{
				Junction::{AccountKey20, GlobalConsensus},
				NetworkId,
			},
			junctions::{Junctions, Junctions::Here},
			location::Location,
		},
	},
};
use subxt::{
	tx::{PairSigner, Payload},
	utils::{AccountId32, MultiAddress},
	OnlineClient,
};

pub fn weth_location() -> Location {
	Location {
		parents: 2,
		interior: Junctions::X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	}
}

pub fn eth_location() -> Location {
	Location {
		parents: 2,
		interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	}
}

pub fn dot_location() -> Location {
	Location { parents: 1, interior: Here }
}

pub async fn mint_eth(asset_hub_client: &Box<OnlineClient<AssetHubConfig>>) {
	let foreign_assets_api =
		crate::parachains::assethub::api::foreign_assets::calls::TransactionApi;

	// Mint eth to sovereign account
	let admin = MultiAddress::Id(SNOWBRIDGE_SOVEREIGN.into());
	let mut encoded_mint_call = Vec::new();
	foreign_assets_api
		.mint(eth_location(), admin.clone(), 3_500_000_000_000)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_mint_call)
		.expect("encoded call");
	snowbridge_assethub_call_from_relay_chain(encoded_mint_call)
		.await
		.expect("fund snowbridge sovereign with eth for pool");
}

pub async fn create_asset_pool(asset_hub_client: &Box<OnlineClient<AssetHubConfig>>) {
	// Check if the pool has been created. The storage lookup for the pool did not work,
	// so checking if the pool ID has been incremented as an indication that the pool has been
	// created.
	let next_id = asset_hub_client
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&assethub::api::storage().asset_conversion().next_pool_asset_id())
		.await
		.unwrap();

	if next_id.is_some() && next_id.unwrap() > 0 {
		println!("Pool has already been created, skipping.");
		return;
	}

	let foreign_assets_api =
		crate::parachains::assethub::api::foreign_assets::calls::TransactionApi;

	// Mint eth to sovereign account
	println!("Minting eth to Snowbridge sovereign.");
	let admin = MultiAddress::Id(SNOWBRIDGE_SOVEREIGN.into());
	let mut encoded_mint_call = Vec::new();
	foreign_assets_api
		.mint(eth_location(), admin.clone(), 3_500_000_000_000)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_mint_call)
		.expect("encoded call");
	snowbridge_assethub_call_from_relay_chain(encoded_mint_call)
		.await
		.expect("fund snowbridge sovereign with eth for pool");

	// Transfer funds to Ferdie, who will create the pool
	println!("Transferring funds to Ferdie to create the pool.");
	let ferdie_account: AccountId32 = (*FERDIE_PUBLIC).into();
	let mut encoded_create_pool_call = Vec::new();
	foreign_assets_api
		.transfer(eth_location(), MultiAddress::Id(ferdie_account.clone()), 3_000_000_000_000)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_create_pool_call)
		.expect("encoded call");
	snowbridge_assethub_call_from_relay_chain(encoded_create_pool_call)
		.await
		.expect("transfer eth to ferdie");

	// Create the pool
	println!("Creating the pool.");
	let create_pool_call = assethub::api::tx()
		.asset_conversion()
		.create_pool(dot_location(), eth_location());
	let signer: PairSigner<AssetHubConfig, _> = PairSigner::new((*FERDIE).clone());
	asset_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&create_pool_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("pool created");

	//wait_for_assethub_event::<PoolCreated>(asset_hub_client).await;

	// Add liquidity to the pool.
	println!("Adding liquidity.");
	let create_liquidity = assethub::api::tx().asset_conversion().add_liquidity(
		dot_location(),
		eth_location(),
		1_000_000_000_000,
		2_000_000_000_000,
		1,
		1,
		ferdie_account,
	);
	let signer: PairSigner<AssetHubConfig, _> = PairSigner::new((*FERDIE).clone());
	asset_hub_client
		.tx()
		.sign_and_submit_then_watch_default(&create_liquidity, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("liquidity added");

	//wait_for_assethub_event::<LiquidityAdded>(asset_hub_client).await;
}

pub async fn mint_token_to(
	asset_hub_client: &Box<OnlineClient<AssetHubConfig>>,
	token: Location,
	who: [u8; 32],
	amount: u128,
) {
	let foreign_assets_api =
		crate::parachains::assethub::api::foreign_assets::calls::TransactionApi;

	// Mint eth to sovereign account
	let beneficiary = MultiAddress::Id(who.into());
	let mut encoded_mint_call = Vec::new();
	foreign_assets_api
		.mint(token, beneficiary, amount)
		.encode_call_data_to(&asset_hub_client.metadata(), &mut encoded_mint_call)
		.expect("encoded call");
	snowbridge_assethub_call_from_relay_chain(encoded_mint_call)
		.await
		.expect("fund snowbridge sovereign with eth for pool");
}
