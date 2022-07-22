#[cfg(feature = "minimal")]
mod minimal;

#[cfg(not(feature = "minimal"))]
mod mainnet;

#[cfg(feature = "minimal")]
pub use minimal::*;

#[cfg(not(feature = "minimal"))]
pub use mainnet::*;