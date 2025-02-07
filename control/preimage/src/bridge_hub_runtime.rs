#[cfg(feature = "polkadot")]
pub use bridge_hub_polkadot_runtime::runtime_types::bridge_hub_polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use bridge_hub_polkadot_runtime::*;

#[cfg(feature = "westend")]
pub use bridge_hub_westend_runtime::runtime_types::bridge_hub_westend_runtime::RuntimeCall;
#[cfg(feature = "westend")]
pub use bridge_hub_westend_runtime::*;

#[cfg(feature = "paseo")]
pub use bridge_hub_paseo_runtime::runtime_types::bridge_hub_paseo_runtime::RuntimeCall;
#[cfg(feature = "paseo")]
pub use bridge_hub_paseo_runtime::*;
