use alloy::primitives::{Address, FixedBytes};
use assethub::api::polkadot_xcm::calls::TransactionApi;
use codec::Encode;
use hex_literal::hex;
use snowbridge_smoketest::{
	asset_hub_helper::{eth_location, mint_token_to},
	constants::*,
	contracts::{
		hello_world::{HelloWorld, HelloWorld::SaidHello},
		i_gateway_v2 as i_gateway,
	},
	helper::{initial_clients, wait_for_ethereum_event, AssetHubConfig},
	helper_v2::{fund_agent_v2, get_agent_address},
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

const INITIAL_FUND: u128 = 3_000_000_000_000;
const INITIAL_FUND_IN_ETHER: u128 = 1_000_000_000_000_000;
const HELLO_WORLD_CONTRACT: [u8; 20] = hex!("8cf6147918a5cbb672703f879f385036f8793a24");

#[tokio::test]
async fn agent_transact() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_gateway::IGatewayV2::new(gateway_addr, ethereum_client.clone());
	// register agent if not exist
	let agent_address = gateway.agentOf(FixedBytes::from(ASSET_HUB_BOB_AGENT_ID)).call().await;
	if !agent_address.is_ok() {
		gateway
			.v2_createAgent(FixedBytes::from(ASSET_HUB_BOB_AGENT_ID))
			.gas_price(GAS_PRICE)
			.send()
			.await
			.unwrap()
			.get_receipt()
			.await
			.expect("get agent receipt");
	}
	// Initial fund for the AH agent
	fund_agent_v2(ASSET_HUB_AGENT_ID, INITIAL_FUND_IN_ETHER)
		.await
		.expect("fund the agent");

	// Initial fund for the user agent
	let agent_address = get_agent_address(ethereum_client.clone(), ASSET_HUB_BOB_AGENT_ID)
		.await
		.expect("find agent");
	println!("agent address {}", hex::encode(agent_address));
	fund_agent_v2(ASSET_HUB_BOB_AGENT_ID, INITIAL_FUND_IN_ETHER)
		.await
		.expect("fund the agent");

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

	let hello_world = HelloWorld::new(Address::from(HELLO_WORLD_CONTRACT), ethereum_client.clone());
	let encoded_data = hello_world.sayHello("Hello!".to_string()).calldata().to_vec();

	println!("data is {}", hex::encode(encoded_data.clone()));

	let transact_info = ContractCall::V1 {
		target: HELLO_WORLD_CONTRACT,
		calldata: encoded_data,
		gas: 80000,
		value: 1_000_000_000,
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
			key: agent_address.into(),
		}]),
	};

	let local_fee_amount: u128 = 800_000_000_000;
	let local_fee_asset = Asset {
		id: AssetId(Location { parents: 1, interior: Junctions::Here }),
		fun: Fungible(local_fee_amount),
	};
	let amount: u128 = 4_000_000_000;
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

	wait_for_ethereum_event::<SaidHello>(ethereum_client, HELLO_WORLD_CONTRACT.into()).await;
}
