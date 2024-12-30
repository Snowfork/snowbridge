#[cfg(feature = "polkadot")]
pub use asset_hub_polkadot_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use asset_hub_polkadot_runtime::*;

#[cfg(feature = "westend")]
pub use asset_hub_westend_runtime::runtime_types::asset_hub_westend_runtime::RuntimeCall;
#[cfg(feature = "westend")]
pub use asset_hub_westend_runtime::*;

#[cfg(feature = "paseo")]
pub use asset_hub_paseo_runtime::runtime_types::asset_hub_paseo_runtime::RuntimeCall;
#[cfg(feature = "paseo")]
pub use asset_hub_paseo_runtime::*;
