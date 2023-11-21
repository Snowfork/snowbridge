use hex_literal::hex;

pub const ASSET_HUB_PARA_ID: u32 = 1000;
pub const BRIDGE_HUB_PARA_ID: u32 = 1013;
pub const PENPAL_PARA_ID: u32 = 2000;

pub const ETHEREUM_API: &str = "ws://localhost:8546";
pub const ETHEREUM_HTTP_API: &str = "http://localhost:8545";

pub const ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:12144";
pub const BRIDGE_HUB_WS_URL: &str = "ws://127.0.0.1:11144";
pub const PENPAL_WS_URL: &str = "ws://127.0.0.1:14144";
pub const RELAY_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
pub const TEMPLATE_NODE_WS_URL: &str = "ws://127.0.0.1:13144";

pub const ETHEREUM_CHAIN_ID: u64 = 15;
pub const ETHEREUM_KEY: &str = "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
pub const ETHEREUM_ADDRESS: [u8; 20] = hex!("90A987B944Cb1dCcE5564e5FDeCD7a54D3de27Fe");

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify
// the order in contracts are deployed in DeployScript.sol.
pub const GATEWAY_PROXY_CONTRACT: [u8; 20] = hex!("EDa338E4dC46038493b885327842fD3E301CaB39");
pub const WETH_CONTRACT: [u8; 20] = hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");

// Agent for bridge hub parachain 1013
pub const BRIDGE_HUB_AGENT_ID: [u8; 32] =
	hex!("05f0ced792884ed09997292bd95f8d0d1094bb3bded91ec3f2f08531624037d6");
// Agent for asset hub parachain 1000
pub const ASSET_HUB_AGENT_ID: [u8; 32] =
	hex!("72456f48efed08af20e5b317abf8648ac66e86bb90a411d9b0b713f7364b75b4");
// Agent for template parachain 1001
pub const SIBLING_AGENT_ID: [u8; 32] =
	hex!("e01018a3378502770faff44fbef3910d120a0353d18be653625b8daa88a86453");

pub const ASSET_HUB_SOVEREIGN: [u8; 32] =
	hex!("7369626ce8030000000000000000000000000000000000000000000000000000");
pub const SNOWBRIDGE_SOVEREIGN: [u8; 32] =
	hex!("da4d66c3651dc151264eee5460493210338e41a7bbfca91a520e438daf180bf5");
pub const PENPAL_SOVEREIGN: [u8; 32] =
	hex!("7369626cd0070000000000000000000000000000000000000000000000000000");

// SS58: DE14BzQ1bDXWPKeLoAqdLAm1GpyAWaWF1knF74cEZeomTBM
pub const FERDIE: [u8; 32] =
	hex!("1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c");
