use hex::FromHex;
use hex_literal::hex;
use lazy_static::lazy_static;
use std::{env, string::ToString};
use subxt::ext::sp_core::{sr25519::Pair, Pair as PairT};

// Todo: load all configs from env in consistent with set-env.sh
pub const ASSET_HUB_PARA_ID: u32 = 1000;
pub const BRIDGE_HUB_PARA_ID: u32 = 1002;
pub const PENPAL_PARA_ID: u32 = 2000;

pub const DEFAULT_ETHEREUM_API: &str = "ws://localhost:8546";
pub const DEFAULT_ETHEREUM_HTTP_API: &str = "http://localhost:8545";

pub const DEFAULT_BRIDGE_HUB_WS_URL: &str = "ws://127.0.0.1:11144";
pub const DEFAULT_ASSET_HUB_WS_URL: &str = "ws://127.0.0.1:12144";
pub const PENPAL_WS_URL: &str = "ws://127.0.0.1:13144";
pub const DEFAULT_RELAY_CHAIN_WS_URL: &str = "ws://127.0.0.1:9944";
pub const TEMPLATE_NODE_WS_URL: &str = "ws://127.0.0.1:13144";

pub const ETHEREUM_CHAIN_ID: u64 = 11155111;
pub const DEFAULT_ETHEREUM_KEY: &str =
	"0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342";
pub const ETHEREUM_ADDRESS: [u8; 20] = hex!("90A987B944Cb1dCcE5564e5FDeCD7a54D3de27Fe");

// The deployment addresses of the following contracts are stable in our E2E env, unless we modify
// the order in contracts are deployed in DeployScript.sol.
pub const DEFAULT_GATEWAY_PROXY_CONTRACT: [u8; 20] =
	hex!("87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d");
pub const DEFAULT_WETH_CONTRACT: [u8; 20] = hex!("774667629726ec1FaBEbCEc0D9139bD1C8f72a23");
pub const AGENT_EXECUTOR_CONTRACT: [u8; 20] = hex!("Fc97A6197dc90bef6bbEFD672742Ed75E9768553");

pub const ERC20_DOT_CONTRACT: [u8; 20] = hex!("B8C39CbCe8106c8415472e3AAe88Eb694Cc70B57");
pub const ERC20_DOT_TOKEN_ID: [u8; 32] =
	hex!("fb3d635c7cb573d1b9e9bff4a64ab4f25190d29b6fd8db94c605a218a23fa9ad");

// Agent for bridge hub parachain 1002
pub const BRIDGE_HUB_AGENT_ID: [u8; 32] =
	hex!("03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314");
// Agent for asset hub parachain 1000
pub const ASSET_HUB_AGENT_ID: [u8; 32] =
	hex!("81c5ab2571199e3188135178f3c2c8e2d268be1313d029b30f534fa579b69b79");
// Agent for penpal parachain 2000
pub const SIBLING_AGENT_ID: [u8; 32] =
	hex!("5097ee1101e90c3aadb882858c59a22108668021ec81bce9f4930155e5c21e59");

pub const ASSET_HUB_SOVEREIGN: [u8; 32] =
	hex!("7369626ce8030000000000000000000000000000000000000000000000000000");
pub const SNOWBRIDGE_SOVEREIGN: [u8; 32] =
	hex!("ce796ae65569a670d0c1cc1ac12515a3ce21b5fbf729d63d7b289baad070139d");
pub const PENPAL_SOVEREIGN: [u8; 32] =
	hex!("7369626cd0070000000000000000000000000000000000000000000000000000");

lazy_static! {
	// SS58: 5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL
	pub static ref ALICE: Pair = Pair::from_string("//Alice", None)
		.expect("cannot create keypair");
	pub static ref FERDIE: Pair = Pair::from_string("//Ferdie", None)
		.expect("cannot create keypair");
	pub static ref FERDIE_PUBLIC: [u8; 32] = (*FERDIE).public().into();
	// SS58: 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
	pub static ref BOB: Pair = Pair::from_string("//Bob", None)
		.expect("cannot create keypair");
	pub static ref BOB_PUBLIC: [u8; 32] = (*BOB).public().into();

	pub static ref REGISTER_TOKEN_FEE: u64 = env::var("REGISTER_TOKEN_FEE")
		.unwrap_or("200000000000000000".to_string())
		.parse()
		.unwrap();
	pub static ref CREATE_ASSET_FEE: u128 = env::var("CREATE_ASSET_FEE")
		.unwrap_or("10000000000000".to_string())
		.parse()
		.unwrap();
	pub static ref RESERVE_TRANSFER_FEE: u128 = env::var("RESERVE_TRANSFER_FEE")
		.unwrap_or("20000000000".to_string())
		.parse()
		.unwrap();
	pub static ref EXCHANGE_RATE: u128 = env::var("EXCHANGE_RATE")
		.unwrap_or("2500000000000000".to_string())
		.parse()
		.unwrap();
	pub static ref FEE_MULTIPLIER: u128 = env::var("FEE_MULTIPLIER")
		.unwrap_or("1000000000000000000".to_string())
		.parse()
		.unwrap();
	pub static ref FEE_PER_GAS: u64 =
		env::var("FEE_PER_GAS").unwrap_or("20000000000".to_string()).parse().unwrap();
	pub static ref LOCAL_REWARD: u128 =
		env::var("LOCAL_REWARD").unwrap_or("1000000000000".to_string()).parse().unwrap();
	pub static ref REMOTE_REWARD: u64 = env::var("REMOTE_REWARD")
		.unwrap_or("1000000000000000".to_string())
		.parse()
		.unwrap();
	pub static ref BRIDGE_HUB_WS_URL: String = {
		if let Ok(val) = env::var("BRIDGE_HUB_WS_URL") {
				val
		}
		else {
			DEFAULT_BRIDGE_HUB_WS_URL.to_string()
		}
	};
	pub static ref ASSET_HUB_WS_URL: String = {
		if let Ok(val) = env::var("ASSET_HUB_WS_URL") {
				val
		}
		else {
			DEFAULT_ASSET_HUB_WS_URL.to_string()
		}
	};
	pub static ref RELAY_CHAIN_WS_URL: String = {
		if let Ok(val) = env::var("RELAY_CHAIN_WS_URL") {
				val
		}
		else {
			DEFAULT_RELAY_CHAIN_WS_URL.to_string()
		}
	};
	pub static ref ETHEREUM_API: String = {
		if let Ok(val) = env::var("ETHEREUM_API") {
				val
		}
		else {
			DEFAULT_ETHEREUM_API.to_string()
		}
	};
	pub static ref ETHEREUM_HTTP_API: String = {
		if let Ok(val) = env::var("ETHEREUM_HTTP_API") {
				val
		}
		else {
			DEFAULT_ETHEREUM_HTTP_API.to_string()
		}
	};
	pub static ref ETHEREUM_KEY: String = {
		if let Ok(val) = env::var("ETHEREUM_KEY") {
				val
		}
		else {
			DEFAULT_ETHEREUM_KEY.to_string()
		}
	};
	pub static ref GATEWAY_PROXY_CONTRACT: [u8; 20] = {
		if let Ok(val) = env::var("GATEWAY_PROXY_CONTRACT") {
				<[u8; 20]>::from_hex(val.strip_prefix("0x").unwrap_or(&val)).unwrap()
		}
		else {
			DEFAULT_GATEWAY_PROXY_CONTRACT
		}
	};
	pub static ref WETH_CONTRACT: [u8; 20] = {
		if let Ok(val) = env::var("WETH_CONTRACT") {
				<[u8; 20]>::from_hex(val).unwrap()
		}
		else {
			DEFAULT_WETH_CONTRACT
		}
	};

	pub static ref SUBSTRATE_RECEIVER: [u8; 32] = {
		if let Ok(val) = env::var("SUBSTRATE_RECEIVER") {
				<[u8; 32]>::from_hex(val).unwrap()
		}
		else {
			BOB_PUBLIC.clone()
		}
	};

	pub static ref ETHEREUM_RECEIVER: [u8; 20] = {
		if let Ok(val) = env::var("ETHEREUM_RECEIVER") {
				<[u8; 20]>::from_hex(val).unwrap()
		}
		else {
			<[u8; 20]>::from_hex("44a57ee2f2FCcb85FDa2B0B18EBD0D8D2333700e").unwrap()
		}
	};
	pub static ref SUBSTRATE_KEY: String = {
		if let Ok(val) = env::var("SUBSTRATE_KEY") {
				"0x".to_owned() + &val
		}
		else {
			"//Bob".to_string()
		}
	};
	pub static ref WAIT_PERIOD: u64 =
		env::var("WAIT_PERIOD").unwrap_or("100".to_string()).parse().unwrap();
}
