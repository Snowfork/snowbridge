use assethub::api::polkadot_xcm::calls::TransactionApi;
use snowbridge_smoketest::{
	asset_hub_helper::{eth_location, mint_token_to},
	constants::*,
	contracts::token::Token::Transfer,
	helper::{initial_clients, wait_for_ethereum_event, AssetHubConfig},
	parachains::assethub::{
		api::runtime_types::{
			bounded_collections::bounded_vec::BoundedVec,
			sp_weights::weight_v2::Weight,
			staging_xcm::v5::{
				asset::{
					Asset,
					AssetFilter::{Definite, Wild},
					AssetId, AssetTransferFilter, Assets,
					Fungibility::Fungible,
					WildAsset::AllCounted,
				},
				junction::{
					Junction::{AccountKey20, GlobalConsensus},
					NetworkId,
				},
				junctions::Junctions,
				location::Location,
				Instruction::*,
				Xcm,
			},
			xcm::VersionedXcm,
		},
		{self},
	},
};
use std::str::FromStr;
use subxt::OnlineClient;
use subxt_signer::{
	sr25519::{self},
	SecretUri,
};

const INITIAL_FUND: u128 = 3_000_000_000_000;
#[tokio::test]
async fn transfer_pna() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let suri = SecretUri::from_str(&SUBSTRATE_KEY).expect("Parse SURI");
	let signer = sr25519::Keypair::from_uri(&suri).expect("valid keypair");
	// Mint ether to sender to pay fees
	mint_token_to(
		&test_clients.asset_hub_client,
		eth_location(),
		signer.public_key().0,
		INITIAL_FUND,
	)
	.await;

	let assethub: OnlineClient<AssetHubConfig> = *test_clients.asset_hub_client;

	let destination = Location {
		parents: 2,
		interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	};

	let beneficiary = Location {
		parents: 0,
		interior: Junctions::X1([AccountKey20 { network: None, key: ETHEREUM_ADDRESS.into() }]),
	};

	// Local fee amount(in DOT) should cover
	// 1. execution cost on AH
	// 2. delivery cost to BH
	// 3. execution cost on BH
	let local_fee_amount = 200_000_000_000;
	// Remote fee amount(in WETH) should cover execution cost on Ethereum
	let remote_fee_amount = 4_000_000_000;

	const TOKEN_AMOUNT: u128 = 100_000_000_000;

	let fee_asset_location: Location = Location {
		parents: 2,
		interior: Junctions::X1([GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	};

	let local_fee_asset = Asset {
		id: AssetId(Location { parents: 1, interior: Junctions::Here }),
		fun: Fungible(local_fee_amount),
	};
	let remote_fee_asset =
		Asset { id: AssetId(fee_asset_location), fun: Fungible(remote_fee_amount) };

	let assets = vec![
		Asset {
			id: AssetId(Location { parents: 1, interior: Junctions::Here }),
			fun: Fungible(TOKEN_AMOUNT + local_fee_amount),
		},
		remote_fee_asset.clone(),
	];

	let xcm = VersionedXcm::V5(Xcm(vec![
		WithdrawAsset(Assets(assets.into())),
		PayFees { asset: local_fee_asset },
		InitiateTransfer {
			destination,
			remote_fees: Some(AssetTransferFilter::ReserveWithdraw(Definite(Assets(
				vec![remote_fee_asset.clone()].into(),
			)))),
			preserve_origin: true,
			assets: BoundedVec(vec![AssetTransferFilter::ReserveDeposit(Definite(Assets(vec![
				Asset {
					id: AssetId(Location { parents: 1, interior: Junctions::Here }),
					fun: Fungible(TOKEN_AMOUNT),
				},
			])))]),
			remote_xcm: Xcm(vec![DepositAsset { assets: Wild(AllCounted(2)), beneficiary }]),
		},
	]));

	let token_transfer_call =
		TransactionApi.execute(xcm, Weight { ref_time: 8_000_000_000, proof_size: 80_000 });

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success");

	wait_for_ethereum_event::<Transfer>(test_clients.ethereum_client, (*ERC20_DOT_CONTRACT).into())
		.await;
}
