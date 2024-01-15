#[cfg(feature = "beacon-spec-minimal")]
mod ethereum_client_minimal;
#[cfg(feature = "beacon-spec-minimal")]
pub use ethereum_client_minimal::*;

#[cfg(not(feature = "beacon-spec-minimal"))]
mod ethereum_client_mainnet;
#[cfg(not(feature = "beacon-spec-minimal"))]
pub use ethereum_client_mainnet::*;
