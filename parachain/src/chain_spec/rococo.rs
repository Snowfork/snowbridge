use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use rococo_runtime::{
	AccountId, EthereumHeader,
	BalancesConfig, GenesisConfig,
	SystemConfig, VerifierLightclientConfig,
	BasicInboundChannelConfig, IncentivizedInboundChannelConfig,
	ETHConfig, ERC20Config, DOTConfig,
	AssetsConfig, NftConfig,
	CommitmentsConfig,
	ParachainInfoConfig,
	WASM_BINARY, Signature,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public, U256};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

use artemis_core::AssetId;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	let mut props = Properties::new();
	props.insert("tokenSymbol".into(), "DEV".into());
	props.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		"Artemis Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Relay"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				para_id
			)
		},
		vec![],
		None,
		None,
		Some(props),
		Extensions {
			relay_chain: "local_testnet".into(),
			para_id: para_id.into(),
		},
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	endowed_accounts: Vec<AccountId>,
	para_id: ParaId
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		basic_channel_inbound: Some(BasicInboundChannelConfig {
			source_channel: hex!["2ffa5ecdbe006d30397c7636d3e015eee251369f"].into(),
		}),
		incentivized_channel_inbound: Some(IncentivizedInboundChannelConfig {
			source_channel: hex!["eda338e4dc46038493b885327842fd3e301cab39"].into(),
		}),
		assets: Some(AssetsConfig {
			balances: vec![
				(
					AssetId::ETH,
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					U256::from_str_radix("1000000000000000000", 10).unwrap()
				)
			]
		}),
		nft: Some(NftConfig {
			tokens: vec![]
		}),
		verifier_lightclient: Some(VerifierLightclientConfig {
			initial_header: EthereumHeader {
				parent_hash: hex!("3be6a44fc5933721d257099178fa7c228fc74f1870e61bb074047eda1021d2cd").into(),
				timestamp: 1609259210u64.into(),
				number: 11550000u64.into(),
				author: hex!("3ecef08d0e2dad803847e052249bb4f8bff2d5bb").into(),
				transactions_root: hex!("d0265030710d32f7b0b7b20dbe8ca047c1cf1aa8d78b484f0534694eba85bc54").into(),
				ommers_hash: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
				extra_data: hex!("73656f35").into(),
				state_root: hex!("b17f72d61cc7dbd9862a2e7a0ca63268cb21a3e9ca6c895011502607f19ac7f1").into(),
				receipts_root: hex!("9ed944cc02ace88e295db6fb85c8532fa444e6a4ed8a8b618d384dad0d3646bc").into(),
				logs_bloom: (&hex!("19b343276249849050a087e0a20b7b059020be00215c22089409b112fada06b0cc9c714c2d600440c89d5a00da704d1d46da64004daf5b55c551dee6c37111e21119a1e09b42eb72df83622dd43864a89e093f4850d6020414cda740d2e211d1df008882aac08000013cd589b1bea9c046c203692c7894841012cc1b3001dbf85b1c94138374752151c4045cc5264aa210024e915141c2ac482251c4a6158174a3dd8140b8572015b211c1a59b98843103150c0a61a10d22123727e9da284463180c4222a90428247d216f24c7d99c1c040082e3d54745121a183a42ca0828a921b13dfc3c0b4460914035540290fea55c33229a8243045c8c349acd403934b4")).into(),
				gas_used: 0xbe4f11.into(),
				gas_limit: 0xbe8c43.into(),
				difficulty: 0xda5fc499815fau64.into(),
				seal: vec![
					vec![ 160, 3, 99, 254, 41, 148, 9, 136, 202, 4, 55, 19, 132, 10, 201, 17, 179, 47, 42, 203, 77, 1, 14, 85, 150, 63, 45, 32, 29, 121, 249, 171, 87 ],
					vec![ 136, 138, 229, 192, 112, 137, 44, 183, 12 ],
				],
			},
			initial_difficulty: 19755084633726428633088u128.into(),
		}),
		commitments: Some(CommitmentsConfig {
			interval: 1,
		}),
		eth_app: Some(ETHConfig {
			address: hex!["774667629726ec1fabebcec0d9139bd1c8f72a23"].into()
		}),
		erc20_app: Some(ERC20Config {
			address: hex!["83428c7db9815f482a39a1715684dCF755021997"].into()
		}),
		dot_app: Some(DOTConfig {
			address: hex!["b1185ede04202fe62d38f5db72f71e38ff3e8305"].into()
		}),
		erc721_app: Some(ERC721Config {
			// TODO: fill proper address
			address: hex!["b1185ede04202fe62d38f5db72f71e38ff3e8305"].into()
		}),
		parachain_info: Some(ParachainInfoConfig { parachain_id: para_id }),
	}
}
