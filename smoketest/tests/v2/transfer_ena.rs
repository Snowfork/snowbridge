use assethub::api::polkadot_xcm::calls::TransactionApi;
use snowbridge_smoketest::{
	asset_hub_helper::{eth_location, mint_token_to},
	constants::*,
	contracts::i_gateway_v2::{IGatewayV2, IGatewayV2::InboundMessageDispatched},
	helper::{initial_clients, wait_for_ethereum_event, AssetHubConfig},
	parachains::assethub::{
		self,
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
				junction::{Junction, NetworkId},
				junctions::Junctions,
				location::Location,
				Instruction::{DepositAsset, InitiateTransfer, PayFees, WithdrawAsset},
				Xcm,
			},
			xcm::VersionedXcm,
		},
	},
};
use std::str::FromStr;
use subxt::OnlineClient;
use subxt_signer::{sr25519, SecretUri};

use alloy::primitives::{Address, FixedBytes};

const INITIAL_FUND: u128 = 3_000_000_000_000;

#[tokio::test]
async fn transfer_ena() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGatewayV2::new(gateway_addr, test_clients.ethereum_client.clone());

	let agent_src = gateway
		.agentOf(FixedBytes::from(ASSET_HUB_AGENT_ID))
		.call()
		.await
		.expect("could not get agent address");
	println!("agent_src: {:?}", agent_src);

	let assethub: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string()).await.unwrap();

	let destination = Location {
		parents: 2,
		interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	};

	let beneficiary = Location {
		parents: 0,
		interior: Junctions::X1([Junction::AccountKey20 {
			network: None,
			key: (*ETHEREUM_RECEIVER).into(),
		}]),
	};

	let local_fee_amount: u128 = 800_000_000_000;
	let local_fee_asset = Asset {
		id: AssetId(Location { parents: 1, interior: Junctions::Here }),
		fun: Fungible(local_fee_amount),
	};
	let amount: u128 = 1_000_000_000;
	let asset_location = Location {
		parents: 2,
		interior: Junctions::X1([Junction::GlobalConsensus(NetworkId::Ethereum {
			chain_id: ETHEREUM_CHAIN_ID,
		})]),
	};
	let remote_fee_asset = Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount / 2) };
	let reserved_asset = Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount / 2) };

	let assets = vec![
		local_fee_asset.clone(),
		Asset { id: AssetId(asset_location.clone()), fun: Fungible(amount) },
	];

	let xcm = VersionedXcm::V5(Xcm(vec![
		WithdrawAsset(Assets(assets.into())),
		PayFees { asset: local_fee_asset.clone() },
		InitiateTransfer {
			destination,
			remote_fees: Some(AssetTransferFilter::ReserveWithdraw(Definite(Assets(
				vec![remote_fee_asset.clone()].into(),
			)))),
			preserve_origin: true,
			assets: BoundedVec(vec![AssetTransferFilter::ReserveWithdraw(Definite(Assets(vec![
				reserved_asset.clone(),
			])))]),
			remote_xcm: Xcm(vec![DepositAsset { assets: Wild(AllCounted(2)), beneficiary }]),
		},
	]));

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

	let token_transfer_call =
		TransactionApi.execute(xcm, Weight { ref_time: 8_000_000_000, proof_size: 80_000 });

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&token_transfer_call, &signer)
		.await
		.expect("call success");

	wait_for_ethereum_event::<InboundMessageDispatched>(test_clients.ethereum_client, gateway_addr)
		.await;
}
