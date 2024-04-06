#[cfg(feature = "rococo")]
pub const POLKADOT_DECIMALS: u8 = 12;

#[cfg(feature = "kusama")]
pub const POLKADOT_DECIMALS: u8 = 12;

#[cfg(feature = "polkadot")]
pub const POLKADOT_DECIMALS: u8 = 10;

pub const ETHER_DECIMALS: u8 = 18;

pub const GWEI_UNIT: u128 = 1_000_000_000;

#[cfg(feature = "rococo")]
pub const POLKADOT_SYMBOL: &str = "ROC";

#[cfg(feature = "kusama")]
pub const POLKADOT_SYMBOL: &str = "KSM";

#[cfg(feature = "polkadot")]
pub const POLKADOT_SYMBOL: &str = "DOT";
