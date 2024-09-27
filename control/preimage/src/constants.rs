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

#[cfg(feature = "westend")]
mod westend {
    pub const POLKADOT_SYMBOL: &str = "WND";
    pub const POLKADOT_DECIMALS: u8 = 12;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const ASSET_HUB_API: &str = "wss://westend-asset-hub-rpc.polkadot.io";
    pub const BRIDGE_HUB_ID: u32 = 1002;
    pub const BRIDGE_HUB_API: &str = "wss://westend-bridge-hub-rpc.polkadot.io";
    pub const RELAY_API: &str = "wss://westend-rpc.polkadot.io";
}

#[cfg(feature = "westend")]
pub use westend::*;

#[cfg(feature = "paseo")]
mod paseo {
    pub const POLKADOT_SYMBOL: &str = "PAS";
    pub const POLKADOT_DECIMALS: u8 = 10;
    pub const ASSET_HUB_ID: u32 = 1000;
    pub const ASSET_HUB_API: &str = "wss://asset-hub-paseo-rpc.dwellir.com";
    pub const BRIDGE_HUB_ID: u32 = 1002;
    pub const BRIDGE_HUB_API: &str = "wss://sys.ibp.network/bridge-hub-paseo";
    pub const RELAY_API: &str = "wss://paseo-rpc.dwellir.com";
}

#[cfg(feature = "paseo")]
pub use paseo::*;
