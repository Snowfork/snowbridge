#[cfg(feature = "rococo")]
pub use rococo_runtime::runtime_types::rococo_runtime::RuntimeCall;
#[cfg(feature = "rococo")]
pub use rococo_runtime::*;

#[cfg(feature = "polkadot")]
pub use polkadot_runtime::api::*;
#[cfg(feature = "polkadot")]
pub use polkadot_runtime::runtime::api::runtime_types::polkadot_runtime::RuntimeCall;
