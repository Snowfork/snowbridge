use crate::{
	constants::*,
	parachains::penpal::{self, api::runtime_types as penpalTypes},
};
use penpalTypes::{
	penpal_runtime::RuntimeCall as PenpalRuntimeCall,
	staging_xcm::v4::{
		junction::Junction as PenpalJunction, junctions::Junctions as PenpalJunctions,
		location::Location as PenpalLocation,
	},
	xcm::{VersionedLocation as PenpalVersionedLocation, VersionedXcm as PenpalVersionedXcm},
};
use subxt::{
	config::DefaultExtrinsicParams,
	ext::sp_core::{sr25519::Pair, Pair as PairT},
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

	let dest = Box::new(PenpalVersionedLocation::V4(PenpalLocation {
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
