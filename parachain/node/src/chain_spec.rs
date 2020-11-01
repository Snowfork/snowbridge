use cumulus_primitives::ParaId;
use artemis_runtime::{
	AccountId, BalancesConfig, GenesisConfig, Signature, SystemConfig,
	ParachainInfoConfig, VerifierConfig, WASM_BINARY,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
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
	pub fn try_get(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Option<&Self> {
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

pub fn get_chain_spec(id: ParaId) -> std::result::Result<ChainSpec, String> {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "DEV".into());
	properties.insert("tokenDecimals".into(), 12.into());

	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				get_account_id_from_seed::<sr25519::Public>("Relay"),
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
					get_account_id_from_seed::<sr25519::Public>("Relay//stash"),
				],
				id,
			)
		},
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		Some(properties),
		Extensions {
			relay_chain: "local_testnet".into(),
			para_id: id.into(),
		},
	))
}

pub fn staging_test_net(id: ParaId) -> std::result::Result<ChainSpec, String> {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "STG".into());
	properties.insert("tokenDecimals".into(), 12.into());

	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Staging Testnet",
		"staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				get_account_id_from_seed::<sr25519::Public>("Relay"),
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Relay")
				],
				id,
			)
		},
		Vec::new(),
		None,
		// Extensions
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	))
}

fn testnet_genesis(
	wasm_binary: &[u8],
	relay_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		verifier: Some(VerifierConfig {
			key: relay_key,
		}),
		parachain_info: Some(ParachainInfoConfig { parachain_id: id }),
	}
}
