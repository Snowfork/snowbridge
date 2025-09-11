use crate::{
	asset_hub_helper::mint_eth,
	constants::*,
	helper::{assethub_deposit_eth_on_penpal_call_from_relay_chain, AssetHubConfig},
	parachains::penpal::{
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
	penpal_helper::penpal::api::asset_conversion::events::{LiquidityAdded, PoolCreated},
};
use futures::StreamExt;
use pair_signer::PairSigner;
use penpalTypes::{
	penpal_runtime::RuntimeCall as PenpalRuntimeCall,
	staging_xcm::v5::{
		junction::Junction as PenpalJunction, junctions::Junctions as PenpalJunctions,
		location::Location as PenpalLocation,
	},
	xcm::{VersionedLocation as PenpalVersionedLocation, VersionedXcm as PenpalVersionedXcm},
};
use sp_core::{sr25519::Pair, Pair as PairT};
use sp_crypto_hashing::twox_128;
use subxt::{
	config::{
		substrate::{AccountId32, MultiAddress},
		DefaultExtrinsicParams,
	},
	events::StaticEvent,
	ext::codec::Encode,
	utils::H256,
	Config, OnlineClient, PolkadotConfig,
};

/// A concrete PairSigner implementation which relies on `sr25519::Pair` for signing
/// and that PolkadotConfig is the runtime configuration.
mod pair_signer {
	use super::*;
	use sp_core::sr25519;
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		MultiSignature as SpMultiSignature,
	};
	use subxt::{
		config::substrate::{AccountId32, MultiSignature},
		tx::Signer,
	};

	/// A [`Signer`] implementation for [`polkadot_sdk::sp_core::sr25519::Pair`].
	#[derive(Clone)]
	pub struct PairSigner {
		account_id: <PenpalConfig as Config>::AccountId,
		signer: sr25519::Pair,
	}

	impl PairSigner {
		/// Creates a new [`Signer`] from an [`sp_core::sr25519::Pair`].
		pub fn new(signer: sr25519::Pair) -> Self {
			let account_id =
				<SpMultiSignature as Verify>::Signer::from(signer.public()).into_account();
			Self {
				// Convert `sp_core::AccountId32` to `subxt::config::substrate::AccountId32`.
				//
				// This is necessary because we use `subxt::config::substrate::AccountId32` and no
				// From/Into impls are provided between `sp_core::AccountId32` because
				// `polkadot-sdk` isn't a direct dependency in subxt.
				//
				// This can also be done by provided a wrapper type around
				// `subxt::config::substrate::AccountId32` to implement such conversions but
				// that also most likely requires a custom `Config` with a separate `AccountId` type
				// to work properly without additional hacks.
				account_id: AccountId32(account_id.into()),
				signer,
			}
		}
	}

	impl Signer<PenpalConfig> for PairSigner {
		fn account_id(&self) -> <PenpalConfig as Config>::AccountId {
			self.account_id.clone()
		}

		fn sign(&self, signer_payload: &[u8]) -> <PenpalConfig as Config>::Signature {
			let signature = self.signer.sign(signer_payload);
			MultiSignature::Sr25519(signature.0)
		}
	}
}

/// Custom config that works with Penpal
pub enum PenpalConfig {}

impl Config for PenpalConfig {
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

	let signer: PairSigner = PairSigner::new(owner);

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
		return
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
	let signer: PairSigner = PairSigner::new((*FERDIE).clone());
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
	let signer: PairSigner = PairSigner::new((*FERDIE).clone());
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
	let existing_asset = penpal::api::storage().foreign_assets().asset(asset.clone());
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
		return
	}

	println!("creating asset {:?} on penpal.", asset);
	let admin = MultiAddress::Id(ASSET_HUB_SOVEREIGN.into());
	let signer: PairSigner = PairSigner::new((*ALICE).clone());

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
	let signer: PairSigner = PairSigner::new((*ALICE).clone());

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
			break
		}
		if substrate_event_found {
			break
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
