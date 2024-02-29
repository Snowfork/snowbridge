use super::{Context, GatewayOperatingModeArg};
use crate::helpers::wrap_calls;

use alloy_primitives::{Address, Bytes, FixedBytes};
use subxt::utils::{H160, H256};

use crate::bridge_hub_runtime::runtime_types::{
    snowbridge_core::outbound::v1::{Initializer, OperatingMode},
    bridge_hub_rococo_runtime::RuntimeCall as BridgeHubRuntimeCall,
    snowbridge_pallet_system,
};

use crate::relay_runtime::runtime_types::polkadot_runtime::RuntimeCall as RelayRuntimeCall;

pub async fn gateway_operating_mode(context: &Context, mode: GatewayOperatingModeArg) -> Result<RelayRuntimeCall, Box<dyn std::error::Error>> {
    let mode = match mode {
        GatewayOperatingModeArg::Normal => OperatingMode::Normal,
        GatewayOperatingModeArg::RejectingOutboundMessages => OperatingMode::RejectingOutboundMessages,
    };

    let call = BridgeHubRuntimeCall::EthereumSystem(snowbridge_pallet_system::pallet::Call::set_operating_mode { mode });
    let call = wrap_calls(context, vec![call]).await?;
    Ok(call)
}

pub async fn upgrade(
    context: &Context,
    logic_address: Address,
    logic_code_hash: FixedBytes<32>,
    initializer: Option<(Bytes, u64)>,
) -> Result<RelayRuntimeCall, Box<dyn std::error::Error>> {
    let call = BridgeHubRuntimeCall::EthereumSystem(
        snowbridge_pallet_system::pallet::Call::upgrade {
            impl_address: H160::from_slice(logic_address.as_slice()),
            impl_code_hash: H256::from_slice(logic_code_hash.as_slice()),
            initializer: initializer.map(|(params, gas)| Initializer {
                params: params.into(),
                maximum_required_gas: gas,
            }),
        }
    );
    let call = wrap_calls(context, vec![call]).await?;
    Ok(call)
}
