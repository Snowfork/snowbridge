#[cfg(feature = "rococo")]
mod rococo {
    pub const POLKADOT_SYMBOL: &str = "ROC";
    pub const POLKADOT_DECIMALS: u8 = 12;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const BRIDGE_HUB_ID: u32 = 1002;
}

#[cfg(feature = "rococo")]
pub use rococo::*;

#[cfg(feature = "polkadot")]
mod polkadot {
    pub const POLKADOT_SYMBOL: &str = "DOT";
    pub const POLKADOT_DECIMALS: u8 = 10;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const BRIDGE_HUB_ID: u32 = 1002;
}

#[cfg(feature = "polkadot")]
pub use polkadot::*;
