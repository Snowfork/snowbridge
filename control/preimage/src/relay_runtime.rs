#[cfg(feature = "polkadot")]
pub use polkadot_runtime::runtime_types::polkadot_runtime::RuntimeCall;
#[cfg(feature = "polkadot")]
pub use polkadot_runtime::*;

#[cfg(feature = "westend")]
pub use westend_runtime::runtime_types::westend_runtime::RuntimeCall;
#[cfg(feature = "westend")]
pub use westend_runtime::*;

#[cfg(feature = "paseo")]
pub use paseo_runtime::runtime_types::paseo_runtime::RuntimeCall;
#[cfg(feature = "paseo")]
pub use paseo_runtime::*;
