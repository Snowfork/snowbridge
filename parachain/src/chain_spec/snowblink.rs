use cumulus_primitives_core::ParaId;
use sc_service::ChainType;
use snowblink_runtime::{AccountId, AuraId, EtherAppPalletId, GenesisConfig, WASM_BINARY};
use sp_core::sr25519;
use sp_runtime::{traits::AccountIdConversion, Perbill};

use super::{get_account_id_from_seed, get_collator_keys_from_seed, Extensions};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub fn get_chain_spec() -> ChainSpec {
	let mut props = sc_chain_spec::Properties::new();
	props.insert("tokenSymbol".into(), "SNO".into());
	props.insert("tokenDecimals".into(), 12u8.into());

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
				1000.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(props),
		Extensions { relay_chain: "rococo-local".into(), para_id: 1000 },
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
			source_channel: Default::default(),
		},
		basic_outbound_channel: snowblink_runtime::BasicOutboundChannelConfig {
			principal: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
			interval: 1,
		},
		incentivized_inbound_channel: snowblink_runtime::IncentivizedInboundChannelConfig {
			source_channel: Default::default(),
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
		ethereum_light_client: snowblink_runtime::EthereumLightClientConfig {
			initial_header: Default::default(),
			initial_difficulty: Default::default(),
		},
		dot_app: snowblink_runtime::DotAppConfig {
			address: Default::default(),
		},
		eth_app: snowblink_runtime::EthAppConfig {
			address: Default::default(),
		},
		erc_20_app: snowblink_runtime::Erc20AppConfig {
			address: Default::default(),
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
