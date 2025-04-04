use crate::{
	asset_hub_helper::mint_eth,
	constants::*,
	helper::{
		assethub_deposit_eth_on_penpal_call_from_relay_chain, initial_clients, AssetHubConfig,
	},
	parachains::{
		penpal::{
			self,
			api::{
				runtime_types as penpalTypes,
				runtime_types::staging_xcm::v5::{
					junction::{
						Junction::{AccountKey20, GlobalConsensus},
						NetworkId,
					},
					junctions::{Junctions, Junctions::Here},
				},
			},
		},
		relaychain,
		relaychain::api::runtime_types::{
			pallet_xcm::pallet::Call as RelaychainPalletXcmCall,
			sp_weights::weight_v2::Weight as RelaychainWeight,
			staging_xcm::v3::multilocation::MultiLocation as RelaychainMultiLocation,
			westend_runtime::RuntimeCall as RelaychainRuntimeCall,
			xcm::{
				v3::{
					junction::{
						Junction as RelaychainJunction,
						Junction::AccountId32 as RelaychainAccountId32,
						NetworkId as RelaychainNetworkId,
					},
					junctions::Junctions as RelaychainJunctions,
					multiasset::{
						AssetId as RelaychainAssetId, Fungibility as RelaychainFungibility,
						MultiAsset as RelaychainMultiAsset,
						MultiAssetFilter as RelaychainMultiAssetFilter,
						MultiAssets as RelaychainMultiAssets,
						WildMultiAsset as RelaychainWildMultiAsset,
					},
					Instruction as RelaychainInstruction, WeightLimit as RelaychainWeightLimit,
					Xcm as RelaychainXcm,
				},
				VersionedLocation as RelaychainVersionedLocation,
				VersionedXcm as RelaychainVersionedXcm,
			},
		},
	},
	penpal_helper::penpal::api::asset_conversion::events::{LiquidityAdded, PoolCreated},
};
use futures::StreamExt;
use penpalTypes::{
	penpal_runtime::RuntimeCall as PenpalRuntimeCall,
	staging_xcm::v5::{
		junction::Junction as PenpalJunction, junctions::Junctions as PenpalJunctions,
		location::Location as PenpalLocation,
	},
	xcm::{VersionedLocation as PenpalVersionedLocation, VersionedXcm as PenpalVersionedXcm},
};
use sp_crypto_hashing::twox_128;
use subxt::{
	config::{
		substrate::{AccountId32, MultiAddress},
		DefaultExtrinsicParams,
	},
	events::StaticEvent,
	ext::{
		codec::Encode,
		sp_core::{sr25519::Pair, Pair as PairT},
	},
	tx::PairSigner,
	utils::H256,
	Config, OnlineClient, PolkadotConfig,
};

/// Custom config that works with Penpal
pub enum PenpalConfig {}

impl Config for PenpalConfig {
	type Hash = <PolkadotConfig as Config>::Hash;
	type AccountId = <PolkadotConfig as Config>::AccountId;
	type Address = <PolkadotConfig as Config>::Address;
	type AssetId = <PolkadotConfig as Config>::AssetId;
	type Signature = <PolkadotConfig as Config>::Signature;
	type Hasher = <PolkadotConfig as Config>::Hasher;
	type Header = <PolkadotConfig as Config>::Header;
	type ExtrinsicParams = DefaultExtrinsicParams<PenpalConfig>;
}

pub struct SudoResult {
	pub block_hash: H256,
	pub extrinsic_hash: H256,
}

pub async fn send_sudo_xcm_transact(
	message: Box<PenpalVersionedXcm>,
) -> Result<SudoResult, Box<dyn std::error::Error>> {
	let penpal_client: OnlineClient<PenpalConfig> = OnlineClient::from_url(PENPAL_WS_URL)
		.await
		.expect("can not connect to penpal parachain");

	let dest = Box::new(PenpalVersionedLocation::V5(PenpalLocation {
		parents: 1,
		interior: PenpalJunctions::X1([PenpalJunction::Parachain(BRIDGE_HUB_PARA_ID)]),
	}));

	let sudo_call = penpal::api::sudo::calls::TransactionApi::sudo(
		&penpal::api::sudo::calls::TransactionApi,
		PenpalRuntimeCall::PolkadotXcm(penpalTypes::pallet_xcm::pallet::Call::send {
			dest,
			message,
		}),
	);

	let owner = Pair::from_string("//Alice", None).expect("cannot create keypair");

	let signer: PairSigner<PenpalConfig, _> = PairSigner::new(owner);

	let result = penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through xcm call.")
		.wait_for_finalized()
		.await
		.expect("xcm call failed");

	let block_hash = result.block_hash();
	let extrinsic_hash = result.extrinsic_hash();

	let sudo_result = SudoResult { block_hash, extrinsic_hash };

	if let Err(err) = result.wait_for_success().await {
		Err(Box::new(err))
	} else {
		Ok(sudo_result)
	}
}

pub async fn create_asset_pool(
	penpal_client: &Box<OnlineClient<PenpalConfig>>,
	asset_hub_client: &Box<OnlineClient<AssetHubConfig>>,
) {
	// Check if the pool has been created. The storage lookup for the pool did not work,
	// so checking if the pool ID has been incremented as an indication that the pool has been
	// created.
	let next_id = penpal_client
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&penpal::api::storage().asset_conversion().next_pool_asset_id())
		.await
		.unwrap();

	if next_id.is_some() && next_id.unwrap() > 0 {
		println!("Pool has already been created, skipping.");
		return;
	}

	println!("minting eth on assethub and send to penpal to use for pools");
	mint_eth(&asset_hub_client).await;

	// Transfer funds to Ferdie, who will create the pool.
	let ferdie_account: AccountId32 = (*FERDIE_PUBLIC).into();
	println!("depositing the eth to penpal");
	assethub_deposit_eth_on_penpal_call_from_relay_chain()
		.await
		.expect("transfer eth to ferdie");

	println!("creating the eth/dot asset pool on penpal");
	// Create the pool
	let create_pool_call =
		penpal::api::tx().asset_conversion().create_pool(dot_location(), eth_location());
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*FERDIE).clone());
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&create_pool_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("pool created");

	wait_for_penpal_event::<PoolCreated>(penpal_client).await;

	println!("adding liquidity to the eth/dot asset pool on penpal");
	// Add liquidity to the pool.
	let create_liquidity = penpal::api::tx().asset_conversion().add_liquidity(
		dot_location(),
		eth_location(),
		1_000_000_000_000,
		2_000_000_000_000,
		1,
		1,
		ferdie_account,
	);
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*FERDIE).clone());
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&create_liquidity, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("liquidity added");

	wait_for_penpal_event::<LiquidityAdded>(penpal_client).await;
}

pub async fn ensure_penpal_asset_exists(
	penpal_client: &mut OnlineClient<PenpalConfig>,
	asset: PenpalLocation,
) {
	let existing_asset = penpal::api::storage().foreign_assets().asset(&asset);
	let result = penpal_client
		.storage()
		.at_latest()
		.await
		.unwrap()
		.fetch(&existing_asset)
		.await
		.unwrap();

	if result.is_some() {
		println!("asset {:?} exists on penpal.", asset);
		return;
	}

	println!("creating asset {:?} on penpal.", asset);
	let admin = MultiAddress::Id(ASSET_HUB_SOVEREIGN.into());
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*ALICE).clone());

	let sudo_call = penpal::api::tx().sudo().sudo(PenpalRuntimeCall::ForeignAssets(
		penpalTypes::pallet_assets::pallet::Call2::force_create {
			id: asset,
			owner: admin.clone(),
			is_sufficient: true,
			min_balance: 1,
		},
	));
	penpal_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.unwrap()
		.wait_for_finalized_success()
		.await
		.expect("asset created");
}
pub async fn set_reserve_asset_storage(penpal_client: &mut OnlineClient<PenpalConfig>) {
	use penpal::api::runtime_types::staging_xcm::v5::{
		junction::{Junction::GlobalConsensus, NetworkId},
		junctions::Junctions::X1,
		location::Location,
	};
	let storage_key: Vec<u8> = twox_128(b":CustomizableAssetFromSystemAssetHub:").to_vec();
	let reserve_location: Vec<u8> = Location {
		parents: 2,
		interior: X1([GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID })]),
	}
	.encode();

	println!("setting CustomizableAssetFromSystemAssetHub storage on penpal.");
	let signer: PairSigner<PenpalConfig, _> = PairSigner::new((*ALICE).clone());

	let items = vec![(storage_key, reserve_location)];

	let sudo_call = penpal::api::sudo::calls::TransactionApi::sudo(
		&penpal::api::sudo::calls::TransactionApi,
		PenpalRuntimeCall::System(penpalTypes::frame_system::pallet::Call::set_storage { items }),
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

pub async fn deposit_eth(
	account: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let account_location: RelaychainMultiLocation = RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainAccountId32 {
			network: None,
			id: account.into(),
		}),
	};
	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(PENPAL_PARA_ID)),
	}));

	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::BuyExecution {
			fees: RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 0,
					interior: RelaychainJunctions::Here,
				}),
				fun: RelaychainFungibility::Fungible(amount),
			},
			weight_limit: RelaychainWeightLimit::Limited(RelaychainWeight {
				ref_time: weight,
				proof_size,
			}),
		},
		RelaychainInstruction::ReserveAssetDeposited(RelaychainMultiAssets(vec![
			RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 2,
					interior: RelaychainJunctions::X1(RelaychainJunction::GlobalConsensus(
						RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
					)),
				}),
				fun: RelaychainFungibility::Fungible(amount),
			},
		])),
		RelaychainInstruction::DepositAsset {
			assets: RelaychainMultiAssetFilter::Wild(RelaychainWildMultiAsset::AllCounted(2)),
			beneficiary: account_location,
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn wait_for_penpal_event<Ev: StaticEvent>(
	penpal_client: &Box<OnlineClient<PenpalConfig>>,
) {
	let mut blocks = penpal_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(5);

	let mut substrate_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling penpal block {} for expected event.", block.number());
		let events = block.events().await.expect("read block events");
		for event in events.find::<Ev>() {
			let _ = event.expect("expect upgrade");
			println!(
				"Event found at penpal block {}: {}::{}",
				block.number(),
				<Ev as StaticEvent>::PALLET,
				<Ev as StaticEvent>::EVENT,
			);
			substrate_event_found = true;
			break;
		}
		if substrate_event_found {
			break;
		}
	}
	assert!(substrate_event_found);
}

pub fn weth_location() -> PenpalLocation {
	PenpalLocation {
		parents: 2,
		interior: Junctions::X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID }),
			AccountKey20 { network: None, key: (*WETH_CONTRACT).into() },
		]),
	}
}

pub fn eth_location() -> PenpalLocation {
	PenpalLocation {
		parents: 2,
		interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	}
}

pub fn dot_location() -> PenpalLocation {
	PenpalLocation { parents: 1, interior: Here }
}
