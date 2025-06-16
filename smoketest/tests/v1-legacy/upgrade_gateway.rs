use alloy::{
	dyn_abi::DynSolValue,
	primitives::{keccak256, Address, U256},
	providers::Provider,
};
use futures::StreamExt;
use hex_literal::hex;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_upgradable::IUpgradable::Upgraded, mock_gateway_v2},
	helper::{
		governance_bridgehub_call_from_relay_chain, initial_clients, wait_for_ethereum_event,
	},
	parachains::bridgehub::{
		self,
		api::{
			ethereum_system,
			runtime_types::snowbridge_outbound_queue_primitives::v1::message::Initializer,
		},
	},
};
use subxt::{tx::Payload, OnlineClient, PolkadotConfig};

const GATEWAY_V2_ADDRESS: [u8; 20] = hex!("f8f7758fbcefd546eaeff7de24aff666b6228e73");

#[tokio::test]
async fn upgrade_gateway() {
	let test_clients = initial_clients().await.expect("initialize clients");
	let ethereum_client = test_clients.ethereum_client;

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();

	let new_impl = mock_gateway_v2::MockGatewayV2::new(
		Address::from(GATEWAY_V2_ADDRESS),
		ethereum_client.clone(),
	);
	let new_impl_code = ethereum_client.get_code_at(*new_impl.address()).await.unwrap();
	let new_impl_code_hash = keccak256(new_impl_code);
	let new_impl_initializer_params = DynSolValue::Uint(U256::from(42), 256).abi_encode();

	let bridgehub: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*BRIDGE_HUB_WS_URL).to_string()).await.unwrap();

	let ethereum_system_api = bridgehub::api::ethereum_system::calls::TransactionApi;

	let binding = new_impl.address();
	let address_bytes = binding.as_slice();
	let substrate_address = subxt::utils::H160::from_slice(address_bytes);
	// The upgrade call
	let mut encoded = Vec::new();
	ethereum_system_api
		.upgrade(
			substrate_address,
			(*new_impl_code_hash).into(),
			Some(Initializer {
				params: new_impl_initializer_params,
				maximum_required_gas: 100_000,
			}),
		)
		.encode_call_data_to(&bridgehub.metadata(), &mut encoded)
		.expect("encoded call");

	governance_bridgehub_call_from_relay_chain(encoded)
		.await
		.expect("upgrade contract");

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

	wait_for_ethereum_event::<Upgraded>(ethereum_client, gateway_addr).await;
}
