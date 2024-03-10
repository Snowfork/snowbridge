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

// Weights copied from https://github.com/paritytech/polkadot-sdk/blob/master/cumulus/parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/weights/snowbridge_pallet_outbound_queue.rs
// Added roughly 50%  buffer
pub const PROCESS_MESSAGE_WEIGHT: (u64, u64) = (100_000_000, 5000);
pub const COMMIT_SINGLE_MESSAGE_WEIGHT: (u64, u64) = (20_000_000, 2000);
pub const TOKEN_TRANSFER_GAS_USED_AT_MOST: u64 = 285_000;
