use ethers::prelude::Address;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway, i_gateway::FeeUpdatedFilter},
	helper::*,
	parachains::bridgehub::api::{
		ethereum_control::events::UpdateFees,
		runtime_types::{
			self, bridge_hub_rococo_runtime::RuntimeCall as BHRuntimeCall,
			snowbridge_core::outbound::TokenFees,
		},
	},
};

#[tokio::test]
async fn update_fees() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
	let ethereum_client = *(test_clients.ethereum_client.clone());
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());
	let fees = gateway.asset_fees().await.expect("get fees");
	println!("asset fees {:?}", fees);

	let update_fees_call = BHRuntimeCall::EthereumControl(
		runtime_types::snowbridge_control::pallet::Call::update_fees {
			fees: TokenFees { register: 10_000_000_000_000, send: 20_000_000_000 },
		},
	);

	governance_bridgehub_call_from_relay_chain(vec![update_fees_call])
		.await
		.expect("update fees");

	wait_for_bridgehub_event::<UpdateFees>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<FeeUpdatedFilter>(&test_clients.ethereum_client).await;
}
