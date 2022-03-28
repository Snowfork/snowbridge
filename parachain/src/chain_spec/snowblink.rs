use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::{ChainType, Properties};
use snowblink_runtime::{AccountId, AuraId, EtherAppPalletId, GenesisConfig, WASM_BINARY};
use sp_core::sr25519;
use sp_runtime::{traits::AccountIdConversion, Perbill};

use super::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	let mut props = Properties::new();
	props.insert("tokenSymbol".into(), "ROC".into());
	props.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		"Snowblink Testnet",
		"snowblink_testnet",
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
		system: snowblink_runtime::SystemConfig {
			// Add Wasm runtime to storage.
			code: WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
		},
		balances: snowblink_runtime::BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		sudo: snowblink_runtime::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		local_council: Default::default(),
		local_council_membership: snowblink_runtime::LocalCouncilMembershipConfig {
			members: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
			],
			phantom: Default::default(),
		},
		basic_inbound_channel: snowblink_runtime::BasicInboundChannelConfig {
			source_channel: hex!["B1185EDE04202fE62D38F5db72F71e38Ff3E8305"].into(),
		},
		basic_outbound_channel: snowblink_runtime::BasicOutboundChannelConfig {
			principal: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
			interval: 1,
		},
		incentivized_inbound_channel: snowblink_runtime::IncentivizedInboundChannelConfig {
			source_channel: hex!["8cF6147918A5CBb672703F879f385036f8793a24"].into(),
			reward_fraction: Perbill::from_percent(80),
		},
		incentivized_outbound_channel: snowblink_runtime::IncentivizedOutboundChannelConfig {
			fee: u128::from_str_radix("10000000000000000", 10).unwrap(), // 0.01 SnowEther
			interval: 1,
		},
		assets: snowblink_runtime::AssetsConfig {
			// Initialize the wrapped Ether asset
			assets: vec![(0, EtherAppPalletId::get().into_account(), true, 1)],
			metadata: vec![],
			accounts: vec![],
		},
		asset_registry: snowblink_runtime::AssetRegistryConfig { next_asset_id: 1 },
		nft: snowblink_runtime::NFTConfig { tokens: vec![] },
		ethereum_light_client: snowblink_runtime::EthereumLightClientConfig {
			initial_header: Default::default(),
			initial_difficulty: Default::default(),
		},
		dot_app: snowblink_runtime::DotAppConfig {
			address: hex!["3f839E70117C64744930De8567Ae7A5363487cA3"].into(),
		},
		eth_app: snowblink_runtime::EthAppConfig {
			address: hex!["3f0839385DB9cBEa8E73AdA6fa0CFe07E321F61d"].into(),
		},
		erc_20_app: snowblink_runtime::Erc20AppConfig {
			address: hex!["440eDFFA1352B13227e8eE646f3Ea37456deC701"].into(),
		},
		erc_721_app: snowblink_runtime::Erc721AppConfig {
			address: hex!["F67EFf5250cD974E6e86c9B53dc5290905Bd8916"].into(),
		},
		parachain_info: snowblink_runtime::ParachainInfoConfig { parachain_id: para_id },
		collator_selection: snowblink_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: snowblink_runtime::ExistentialDeposit::get() * 16,
			..Default::default()
		},
		session: snowblink_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                             // account id
						acc,                                     // validator id
						snowblink_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: snowblink_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}
