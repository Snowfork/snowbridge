use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::{ChainType, Properties};
use snowbase_runtime::{AccountId, AuraId, EtherAppPalletId, GenesisConfig, WASM_BINARY};
use sp_core::sr25519;
use sp_runtime::{traits::AccountIdConversion, Perbill};

use super::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	let mut props = Properties::new();
	props.insert("tokenSymbol".into(), "DEV".into());
	props.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		"Snowbase Testnet",
		"snowbase_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
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
				para_id,
			)
		},
		vec![],
		None,
		None,
		None,
		Some(props),
		Extensions { relay_chain: "rococo-local".into(), para_id: para_id.into() },
	)
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	para_id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		system: snowbase_runtime::SystemConfig {
			// Add Wasm runtime to storage.
			code: WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
		},
		balances: snowbase_runtime::BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		sudo: snowbase_runtime::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		local_council: Default::default(),
		local_council_membership: snowbase_runtime::LocalCouncilMembershipConfig {
			members: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
			],
			phantom: Default::default(),
		},
		basic_inbound_channel: snowbase_runtime::BasicInboundChannelConfig {
			source_channel: hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"].into(),
		},
		basic_outbound_channel: snowbase_runtime::BasicOutboundChannelConfig {
			principal: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
			interval: 1,
		},
		incentivized_inbound_channel: snowbase_runtime::IncentivizedInboundChannelConfig {
			source_channel: hex!["EE9170ABFbf9421Ad6DD07F6BDec9D89F2B581E0"].into(),
			reward_fraction: Perbill::from_percent(80),
		},
		incentivized_outbound_channel: snowbase_runtime::IncentivizedOutboundChannelConfig {
			fee: u128::from_str_radix("10000000000000000", 10).unwrap(), // 0.01 SnowEther
			interval: 1,
		},
		assets: snowbase_runtime::AssetsConfig {
			// Initialize the wrapped Ether asset
			assets: vec![(0, EtherAppPalletId::get().into_account(), true, 1)],
			metadata: vec![],
			accounts: vec![],
		},
		asset_registry: snowbase_runtime::AssetRegistryConfig { next_asset_id: 1 },
		nft: snowbase_runtime::NFTConfig { tokens: vec![] },
		ethereum_light_client: snowbase_runtime::EthereumLightClientConfig {
			initial_header: Default::default(),
			initial_difficulty: Default::default(),
		},
		dot_app: snowbase_runtime::DotAppConfig {
			address: hex!["8cF6147918A5CBb672703F879f385036f8793a24"].into(),
		},
		eth_app: snowbase_runtime::EthAppConfig {
			address: hex!["B1185EDE04202fE62D38F5db72F71e38Ff3E8305"].into(),
		},
		erc_20_app: snowbase_runtime::Erc20AppConfig {
			address: hex!["3f0839385DB9cBEa8E73AdA6fa0CFe07E321F61d"].into(),
		},
		erc_721_app: snowbase_runtime::Erc721AppConfig {
			address: hex!["54D6643762E46036b3448659791adAf554225541"].into(),
		},
		parachain_info: snowbase_runtime::ParachainInfoConfig { parachain_id: para_id },
		collator_selection: snowbase_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: snowbase_runtime::ExistentialDeposit::get() * 16,
			..Default::default()
		},
		session: snowbase_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                            // account id
						acc,                                    // validator id
						snowbase_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: snowbase_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}
