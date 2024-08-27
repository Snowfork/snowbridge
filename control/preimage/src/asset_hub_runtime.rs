#[cfg(feature = "rococo")]
pub use asset_hub_rococo_runtime::runtime_types::asset_hub_rococo_runtime::RuntimeCall;
#[cfg(feature = "rococo")]
pub use asset_hub_rococo_runtime::*;

#[cfg(feature = "polkadot")]
pub use asset_hub_polkadot_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use asset_hub_polkadot_runtime::*;

#[cfg(feature = "paseo")]
pub use asset_hub_paseo_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
#[cfg(feature = "paseo")]
pub use asset_hub_paseo_runtime::*;
