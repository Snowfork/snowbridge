#[cfg(feature = "rococo")]
pub use bridge_hub_rococo_runtime::runtime_types::bridge_hub_rococo_runtime::RuntimeCall;
#[cfg(feature = "rococo")]
pub use bridge_hub_rococo_runtime::*;

#[cfg(feature = "polkadot")]
pub use bridge_hub_polkadot_runtime::runtime_types::bridge_hub_polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use bridge_hub_polkadot_runtime::*;

#[cfg(feature = "paseo")]
pub use bridge_hub_paseo_runtime::runtime_types::bridge_hub_paseo_runtime::RuntimeCall;
#[cfg(feature = "paseo")]
pub use bridge_hub_paseo_runtime::*;
