use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use local_runtime::{AccountId, AuraId, EtherAppPalletId, GenesisConfig, Signature, WASM_BINARY};
use sc_service::{ChainType, Properties};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{AccountIdConversion, IdentifyAccount, Verify},
	Perbill,
};

use super::{get_from_seed, Extensions};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	let mut props = Properties::new();
	props.insert("tokenSymbol".into(), "DEV".into());
	props.insert("tokenDecimals".into(), 12.into());

	ChainSpec::from_genesis(
		"Snowbridge Local Testnet",
		"local_testnet",
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
		system: local_runtime::SystemConfig {
			// Add Wasm runtime to storage.
			code: WASM_BINARY.expect("WASM binary was not build, please build it!").to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: local_runtime::BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		sudo: local_runtime::SudoConfig {
			key: get_account_id_from_seed::<sr25519::Public>("Alice"),
		},
		local_council: Default::default(),
		local_council_membership: local_runtime::LocalCouncilMembershipConfig {
			members: vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
			],
			phantom: Default::default(),
		},
		basic_inbound_channel: local_runtime::BasicInboundChannelConfig {
			source_channel: hex!["F8F7758FbcEfd546eAEff7dE24AFf666B6228e73"].into(),
		},
		basic_outbound_channel: local_runtime::BasicOutboundChannelConfig {
			principal: get_account_id_from_seed::<sr25519::Public>("Alice"),
			interval: 1,
		},
		incentivized_inbound_channel: local_runtime::IncentivizedInboundChannelConfig {
			source_channel: hex!["EE9170ABFbf9421Ad6DD07F6BDec9D89F2B581E0"].into(),
			reward_fraction: Perbill::from_percent(80),
		},
		incentivized_outbound_channel: local_runtime::IncentivizedOutboundChannelConfig {
			fee: u128::from_str_radix("10000000000000000", 10).unwrap(), // 0.01 SnowEther
			interval: 1,
		},
		assets: local_runtime::AssetsConfig {
			// Initialize the wrapped Ether asset
			assets: vec![(0, EtherAppPalletId::get().into_account(), true, 1)],
			metadata: vec![],
			accounts: vec![],
		},
		asset_registry: local_runtime::AssetRegistryConfig {
			next_asset_id: 1,
		},
		nft: local_runtime::NFTConfig { tokens: vec![] },
		ethereum_light_client: local_runtime::EthereumLightClientConfig {
			initial_header: Default::default(),
			initial_difficulty: Default::default(),
		},
		dot_app: local_runtime::DotAppConfig {
			address: hex!["8cF6147918A5CBb672703F879f385036f8793a24"].into(),
		},
		eth_app: local_runtime::EthAppConfig {
			address: hex!["B1185EDE04202fE62D38F5db72F71e38Ff3E8305"].into(),
		},
		erc_20_app: local_runtime::Erc20AppConfig {
			address: hex!["3f0839385DB9cBEa8E73AdA6fa0CFe07E321F61d"].into(),
		},
		erc_721_app: local_runtime::Erc721AppConfig {
			address: hex!["54D6643762E46036b3448659791adAf554225541"].into(),
		},
		parachain_info: local_runtime::ParachainInfoConfig { parachain_id: para_id },
		collator_selection: local_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: local_runtime::ExistentialDeposit::get() * 16,
			..Default::default()
		},
		session: local_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                         // account id
						acc,                                 // validator id
						local_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}
