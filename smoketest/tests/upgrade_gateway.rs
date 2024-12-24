use std::{sync::Arc, time::Duration};

use ethers::{
	abi::{Address, Token},
	prelude::*,
	providers::{Provider, Ws},
	utils::keccak256,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	constants::*,
	contracts::{
		i_upgradable::{self, UpgradedFilter},
		mock_gateway_v2,
	},
	parachains::{
		bridgehub::{
			self,
			api::{ethereum_system, runtime_types::snowbridge_core::outbound::v1::Initializer},
		},
		relaychain,
		relaychain::api::runtime_types::{
			pallet_xcm::pallet::Call,
			sp_weights::weight_v2::Weight,
			staging_xcm::v3::multilocation::MultiLocation,
			westend_runtime::RuntimeCall,
			xcm::{
				double_encoded::DoubleEncoded,
				v3::{
					junction::Junction, junctions::Junctions, Instruction, OriginKind, WeightLimit,
					Xcm,
				},
				VersionedLocation, VersionedXcm,
			},
		},
	},
};
use subxt::{
	ext::sp_core::{sr25519::Pair, Pair as PairT},
	tx::{PairSigner, Payload},
	OnlineClient, PolkadotConfig,
};

const GATEWAY_V2_ADDRESS: [u8; 20] = hex!("f8f7758fbcefd546eaeff7de24aff666b6228e73");

#[tokio::test]
async fn upgrade_gateway() {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));
	let ethereum_client = Arc::new(ethereum_provider);

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = i_upgradable::IUpgradable::new(gateway_addr, ethereum_client.clone());

	let new_impl = mock_gateway_v2::MockGatewayV2::new(
		Address::from(GATEWAY_V2_ADDRESS),
		ethereum_client.clone(),
	);
	let new_impl_code = ethereum_client.get_code(new_impl.address(), None).await.unwrap();
	let new_impl_code_hash = keccak256(new_impl_code);
	let new_impl_initializer_params = ethers::abi::encode(&[Token::Uint(42.into())]);

	let relaychain: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*RELAY_CHAIN_WS_URL).to_string()).await.unwrap();
	let bridgehub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*BRIDGE_HUB_WS_URL).to_string()).await.unwrap();

	let sudo: Pair = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner<PolkadotConfig, _> = PairSigner::new(sudo);

	let ethereum_system_api = bridgehub::api::ethereum_system::calls::TransactionApi;

	// The upgrade call
	let mut encoded = Vec::new();
	ethereum_system_api
		.upgrade(
			new_impl.address(),
			new_impl_code_hash.into(),
			Some(Initializer {
				params: new_impl_initializer_params,
				maximum_required_gas: 100_000,
			}),
		)
		.encode_call_data_to(&bridgehub.metadata(), &mut encoded)
		.expect("encoded call");

	let weight = 3000000000;
	let proof_size = 18000;

	let dest = Box::new(VersionedLocation::V3(MultiLocation {
		parents: 0,
		interior: Junctions::X1(Junction::Parachain(BRIDGE_HUB_PARA_ID)),
	}));

	let message = Box::new(VersionedXcm::V3(Xcm(vec![
		Instruction::UnpaidExecution { weight_limit: WeightLimit::Unlimited, check_origin: None },
		Instruction::Transact {
			origin_kind: OriginKind::Superuser,
			require_weight_at_most: Weight { ref_time: weight, proof_size },
			call: DoubleEncoded { encoded },
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api.sudo(RuntimeCall::XcmPallet(Call::send { dest, message }));

	let result = relaychain
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized()
		.await
		.expect("sudo call in block");

	println!("Sudo call issued at relaychain block hash {:?}", result.block_hash());

	result.wait_for_success().await.expect("sudo call success");

	let wait_for_blocks = 5;
	let mut blocks = bridgehub
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(wait_for_blocks);

	let mut upgrade_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling bridgehub block {} for upgrade event.", block.number());
		let upgrades = block.events().await.expect("read block events");
		for upgrade in upgrades.find::<ethereum_system::events::Upgrade>() {
			let _upgrade = upgrade.expect("expect upgrade");
			println!("Event found at bridgehub block {}.", block.number());
			upgrade_event_found = true;
		}
		if upgrade_event_found {
			break;
		}
	}
	assert!(upgrade_event_found);

	let wait_for_blocks = 500;
	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(wait_for_blocks);

	let mut upgrade_event_found = false;
	while let Some(block) = stream.next().await {
		println!("Polling ethereum block {:?} for upgraded event", block.number.unwrap());
		if let Ok(upgrades) = gateway
			.event::<UpgradedFilter>()
			.at_block_hash(block.hash.unwrap())
			.query()
			.await
		{
			for _upgrade in upgrades {
				println!("Upgrade event found at ethereum block {:?}", block.number.unwrap());
				upgrade_event_found = true;
			}
		}
		if upgrade_event_found {
			break;
		}
	}
	assert!(upgrade_event_found);
}
