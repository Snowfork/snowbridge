#[cfg(feature = "rococo")]
mod rococo {
    pub const POLKADOT_SYMBOL: &str = "ROC";
    pub const POLKADOT_DECIMALS: u8 = 12;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const ASSET_HUB_API: &str = "wss://rococo-asset-hub-rpc.polkadot.io";
    pub const BRIDGE_HUB_ID: u32 = 1002;
    pub const BRIDGE_HUB_API: &str = "wss://rococo-bridge-hub-rpc.polkadot.io";
}

#[cfg(feature = "rococo")]
pub use rococo::*;

#[cfg(feature = "polkadot")]
mod polkadot {
    pub const POLKADOT_SYMBOL: &str = "DOT";
    pub const POLKADOT_DECIMALS: u8 = 10;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const ASSET_HUB_API: &str = "wss://polkadot-asset-hub-rpc.polkadot.io";
    pub const BRIDGE_HUB_ID: u32 = 1002;
    pub const BRIDGE_HUB_API: &str = "wss://polkadot-bridge-hub-rpc.polkadot.io";
    pub const RELAY_API: &str = "wss://polkadot.api.onfinality.io/public-ws";
}

#[cfg(feature = "polkadot")]
pub use polkadot::*;
