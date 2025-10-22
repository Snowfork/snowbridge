use alloy::primitives::Address;
use assethub::api::polkadot_xcm::calls::TransactionApi;
use codec::Encode;
use hex_literal::hex;
use snowbridge_smoketest::{
	constants::*,
	contracts::greeter::Greeter,
	helper::{initial_clients, AssetHubConfig},
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
				Instruction::{DepositAsset, InitiateTransfer, PayFees, Transact, WithdrawAsset},
				Xcm,
			},
			xcm::{double_encoded::DoubleEncoded, v3::OriginKind, VersionedXcm},
		},
	},
	types::ContractCall,
};
use std::str::FromStr;
use subxt::OnlineClient;
use subxt_signer::{
	sr25519::{self},
	SecretUri,
};

const L1_GREETER_CONTRACT: [u8; 20] = hex!("31A6Dd971306bb72f2ffF771bF30b1B98dB8B2c5");
const AGENT_CONTRACT: [u8; 20] = hex!("d5B112e020512E1751E10bC74b77954be4d26A60");

#[tokio::test]
async fn transact_p2e_l2() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;

	let signer =
		sr25519::Keypair::from_uri(&SecretUri::from_str(&SUBSTRATE_KEY).expect("Parse SURI"))
			.expect("valid keypair");

	let greeter = Greeter::new(Address::from(L1_GREETER_CONTRACT), ethereum_client.clone());
	let encoded_data = greeter
		.sendGreeting("Hello from end user on Westend!".to_string())
		.calldata()
		.to_vec();

	println!("data is {}", hex::encode(encoded_data.clone()));

	let transact_info = ContractCall::V1 {
		target: L1_GREETER_CONTRACT,
		calldata: encoded_data,
		gas: 1000000,
		value: 0,
	};

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
			key: AGENT_CONTRACT.into(),
		}]),
	};

	let local_fee_amount: u128 = 600_000_000_000;
	let local_fee_asset = Asset {
		id: AssetId(Location { parents: 1, interior: Junctions::Here }),
		fun: Fungible(local_fee_amount),
	};
	let amount: u128 = 800_000_000_000;
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
			remote_xcm: Xcm(vec![
				DepositAsset { assets: Wild(AllCounted(2)), beneficiary },
				Transact {
					origin_kind: OriginKind::SovereignAccount,
					fallback_max_weight: None,
					call: DoubleEncoded { encoded: transact_info.encode() },
				},
			]),
		},
	]));

	let transact_call =
		TransactionApi.execute(xcm, Weight { ref_time: 8_000_000_000, proof_size: 80_000 });

	let _ = assethub
		.tx()
		.sign_and_submit_then_watch_default(&transact_call, &signer)
		.await
		.expect("call success");
}
