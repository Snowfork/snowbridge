pub mod local {
    /// Ethereum chain ID for Sepolia testnet
    pub const CHAIN_ID: u64 = 11155111;

    /// Inbound pallet index for V2 gateway
    pub const INBOUND_PALLET_V2: u8 = 91;

    /// Para ID for Asset Hub
    pub const ASSET_HUB_PARA_ID: u32 = 1000;

    /// Para ID for Bridge Hub
    pub const BRIDGE_HUB_PARA_ID: u32 = 1002;

    /// Deposit required for creating assets
    pub const CREATE_ASSET_DEPOSIT: u128 = 100_000_000_000;

    /// Call index for ForeignAssets::create call
    pub const CREATE_ASSET_CALL: [u8; 2] = [53, 0];

    /// Minimum deposit amount
    pub const MINIMUM_DEPOSIT: u128 = 1;
}

pub mod westend {
    /// Ethereum chain ID for Mainnet
    pub const CHAIN_ID: u64 = 1;

    /// Inbound pallet index for V2 gateway
    pub const INBOUND_PALLET_V2: u8 = 91;

    /// Para ID for Asset Hub
    pub const ASSET_HUB_PARA_ID: u32 = 1000;

    /// Para ID for Bridge Hub
    pub const BRIDGE_HUB_PARA_ID: u32 = 1002;

    /// Deposit required for creating assets
    pub const CREATE_ASSET_DEPOSIT: u128 = 100_000_000_000;

    /// Call index for ForeignAssets::create call
    pub const CREATE_ASSET_CALL: [u8; 2] = [53, 0];

    /// Minimum deposit amount
    pub const MINIMUM_DEPOSIT: u128 = 1;
}

pub mod paseo {
    /// Ethereum chain ID for Mainnet
    pub const CHAIN_ID: u64 = 1;

    /// Inbound pallet index for V2 gateway
    pub const INBOUND_PALLET_V2: u8 = 91;

    /// Para ID for Asset Hub
    pub const ASSET_HUB_PARA_ID: u32 = 1000;

    /// Para ID for Bridge Hub
    pub const BRIDGE_HUB_PARA_ID: u32 = 1002;

    /// Deposit required for creating assets
    pub const CREATE_ASSET_DEPOSIT: u128 = 100_000_000_000;

    /// Call index for ForeignAssets::create call
    pub const CREATE_ASSET_CALL: [u8; 2] = [53, 0];

    /// Minimum deposit amount
    pub const MINIMUM_DEPOSIT: u128 = 1;
}
