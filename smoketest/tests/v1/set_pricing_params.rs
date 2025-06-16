use alloy::primitives::Address;
use snowbridge_smoketest::{
	constants::*,
	contracts::{
		i_gateway_v1,
		i_gateway_v1::IGatewayV1::PricingParametersChanged as EthereumPricingParametersChanged,
	},
	helper::*,
	parachains::{
		bridgehub,
		bridgehub::api::{
			ethereum_system::events::PricingParametersChanged,
			runtime_types::{
				primitive_types::U256,
				snowbridge_core::pricing::{PricingParameters, Rewards},
				sp_arithmetic::fixed_point::FixedU128,
			},
		},
	},
};
use subxt::tx::Payload;

#[tokio::test]
async fn set_pricing_params() {
	let test_clients = initial_clients().await.expect("initialize clients");

	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let ethereum_client = test_clients.ethereum_client;
	let gateway = i_gateway_v1::IGatewayV1::new(gateway_addr, ethereum_client.clone());
	let params = gateway.pricingParameters().call().await.expect("get pricing");
	println!("pricing params {:?}", params);

	let ethereum_system_api = bridgehub::api::ethereum_system::calls::TransactionApi;

	let mut encoded = Vec::new();
	ethereum_system_api
		.set_pricing_parameters(PricingParameters {
			exchange_rate: FixedU128(*EXCHANGE_RATE),
			rewards: Rewards { local: *LOCAL_REWARD, remote: U256([*REMOTE_REWARD, 0, 0, 0]) },
			fee_per_gas: U256([*FEE_PER_GAS, 0, 0, 0]),
			multiplier: FixedU128(*FEE_MULTIPLIER),
		})
		.encode_call_data_to(&test_clients.bridge_hub_client.metadata(), &mut encoded)
		.expect("encoded call");

	governance_bridgehub_call_from_relay_chain(encoded)
		.await
		.expect("set token fees");

	wait_for_bridgehub_event::<PricingParametersChanged>(&test_clients.bridge_hub_client).await;

	wait_for_ethereum_event::<EthereumPricingParametersChanged>(ethereum_client, gateway_addr)
		.await;

	let params = gateway.pricingParameters().call().await.expect("get fees");
	println!("pricing params {:?}", params);
}
