use ethers::prelude::Address;
use snowbridge_smoketest::{
	constants::*,
	contracts::{i_gateway, i_gateway::SetTokenTransferFeesFilter},
	helper::*,
	parachains::bridgehub::api::{
		ethereum_control::events::SetTokenTransferFees,
		runtime_types::{self, bridge_hub_rococo_runtime::RuntimeCall as BHRuntimeCall},
	},
};

#[tokio::test]
async fn set_token_transfer_fees() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let gateway_addr: Address = GATEWAY_PROXY_CONTRACT.into();
	let ethereum_client = *(test_clients.ethereum_client.clone());
	let gateway = i_gateway::IGateway::new(gateway_addr, ethereum_client.clone());
	let fees = gateway.token_transfer_fees().await.expect("get fees");
	println!("asset fees {:?}", fees);

	let set_token_fees_call = BHRuntimeCall::EthereumControl(
		runtime_types::snowbridge_control::pallet::Call::set_token_transfer_fees {
			register: 10_000_000_000_000,
			send: 20_000_000_000,
		},
	);

	governance_bridgehub_call_from_relay_chain(vec![set_token_fees_call])
		.await
		.expect("set token fees");

	wait_for_bridgehub_event::<SetTokenTransferFees>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<SetTokenTransferFeesFilter>(&test_clients.ethereum_client).await;

	let fees = gateway.token_transfer_fees().await.expect("get fees");
	println!("asset fees {:?}", fees);
}
