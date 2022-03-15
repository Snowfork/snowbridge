use crate::cli::{Cli, RelayChainCli, Subcommand};
use codec::Encode;
use cumulus_client_service::genesis::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use log::info;

use crate::chain_spec::Extensions;

#[cfg(feature = "snowbridge-native")]
use crate::chain_spec::snowbridge::{
	get_chain_spec as get_snowbridge_chain_spec, ChainSpec as SnowbridgeChainSpec,
};

#[cfg(feature = "snowblink-native")]
use crate::chain_spec::snowblink::{
	get_chain_spec as get_snowblink_chain_spec, ChainSpec as SnowblinkChainSpec,
};

#[cfg(feature = "snowbase-native")]
use crate::chain_spec::snowbase::{
	get_chain_spec as get_snowbase_chain_spec, ChainSpec as SnowbaseChainSpec,
};

use snowbridge_runtime_primitives::Block;

use polkadot_parachain::primitives::AccountIdConversion;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::Block as BlockT;
use std::{io::Write, net::SocketAddr};

pub type DummyChainSpec = sc_service::GenericChainSpec<(), Extensions>;

const DEFAULT_PARA_ID: u32 = 1000;

trait IdentifyVariant {
	fn is_snowbridge(&self) -> bool;
	fn is_snowblink(&self) -> bool;
	fn is_snowbase(&self) -> bool;
}

impl IdentifyVariant for dyn sc_service::ChainSpec {
	fn is_snowbridge(&self) -> bool {
		self.id().starts_with("snowbridge")
	}
	fn is_snowblink(&self) -> bool {
		self.id().starts_with("snowblink")
	}
	fn is_snowbase(&self) -> bool {
		self.id().starts_with("snowbase")
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyVariant for T {
	fn is_snowbridge(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_snowbridge(self)
	}
	fn is_snowblink(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_snowblink(self)
	}
	fn is_snowbase(&self) -> bool {
		<dyn sc_service::ChainSpec>::is_snowbase(self)
	}
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;

		#[cfg(feature = "snowbridge-native")]
		if runner.config().chain_spec.is_snowbridge() {
			return runner.async_run(|$config| {
				let $components = crate::service::new_partial::<snowbridge_runtime::RuntimeApi, crate::service::SnowbridgeRuntimeExecutor>(
					&$config,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		}

		#[cfg(feature = "snowblink-native")]
		if runner.config().chain_spec.is_snowblink() {
			return runner.async_run(|$config| {
				let $components = crate::service::new_partial::<snowblink_runtime::RuntimeApi, crate::service::SnowblinkRuntimeExecutor>(
					&$config,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		}

		#[cfg(feature = "snowbase-native")]
		if runner.config().chain_spec.is_snowbase() {
			return runner.async_run(|$config| {
				let $components = crate::service::new_partial::<snowbase_runtime::RuntimeApi, crate::service::SnowbaseRuntimeExecutor>(
					&$config,
				)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
		}

		Err(format!("Unknown runtime: {}", runner.config().chain_spec.id()).into())
	}}
}

fn load_spec(
	id: &str,
	para_id: ParaId,
) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	match id {
		#[cfg(feature = "snowbase-native")]
		"snowbase" => Ok(Box::new(get_snowbase_chain_spec(para_id))),
		#[cfg(feature = "snowblink-native")]
		"snowblink" => Ok(Box::new(get_snowblink_chain_spec(para_id))),
		#[cfg(feature = "snowbridge-native")]
		"snowbridge" => Ok(Box::new(get_snowbridge_chain_spec(para_id))),
		path => {
			let path = std::path::PathBuf::from(path);

			let chain_spec = DummyChainSpec::from_json_file(path.clone())?;
			if chain_spec.is_snowbridge() {
				#[cfg(feature = "snowbridge-native")]
				{
					Ok(Box::new(SnowbridgeChainSpec::from_json_file(path.into())?))
				}
				#[cfg(not(feature = "snowbridge-native"))]
				Err(format!(
					"`{}` only supported with `snowbridge-native` feature enabled.",
					chain_spec.id()
				))
			} else if chain_spec.is_snowblink() {
				#[cfg(feature = "snowblink-native")]
				{
					Ok(Box::new(SnowblinkChainSpec::from_json_file(path.into())?))
				}
				#[cfg(not(feature = "snowblink-native"))]
				Err(format!(
					"`{}` only supported with `snowblink-native` feature enabled.",
					chain_spec.id()
				))
			} else {
				#[cfg(feature = "snowbase-native")]
				{
					Ok(Box::new(SnowbaseChainSpec::from_json_file(path.into())?))
				}
				#[cfg(not(feature = "snowbase-native"))]
				Err(format!(
					"`{}` only supported with `snowbase-native` feature enabled.",
					chain_spec.id()
				))
			}
		},
	}
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Snowbridge Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Snowbridge test parachain collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		{} [parachain-args] -- [relaychain-args]",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"http://snowbridge.snowfork.com".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id, self.run.parachain_id.unwrap_or(DEFAULT_PARA_ID).into())
	}

	fn native_runtime_version(
		chain_spec: &Box<dyn sc_service::ChainSpec>,
	) -> &'static RuntimeVersion {
		if chain_spec.is_snowbridge() {
			#[cfg(not(feature = "snowbridge-native"))]
			panic!("`snowbridge-native` feature is not enabled");
			#[cfg(feature = "snowbridge-native")]
			return &snowbridge_runtime::VERSION
		} else if chain_spec.is_snowblink() {
			#[cfg(not(feature = "snowblink-native"))]
			panic!("`snowblink-native` feature is not enabled");
			#[cfg(feature = "snowblink-native")]
			return &snowblink_runtime::VERSION
		} else {
			#[cfg(not(feature = "snowbase-native"))]
			panic!("`snowbase-native` feature is not enabled");
			#[cfg(feature = "snowbase-native")]
			return &snowbase_runtime::VERSION
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Snowbridge Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Parachain collator\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relaychain node.\n\n\
		snowbridge-collator [parachain-args] -- [relaychain-args]"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"http://www.snowfork.com".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name().to_string()].iter())
			.load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn sc_cli::ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> Result<Vec<u8>> {
	let mut storage = chain_spec.build_storage()?;

	storage
		.top
		.remove(sp_core::storage::well_known_keys::CODE)
		.ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name().to_string()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend))
			})
		},
		Some(Subcommand::ExportGenesisState(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let spec =
				load_spec(&params.chain.clone().unwrap_or_default(), params.parachain_id.into())?;
			let state_version = Cli::native_runtime_version(&spec).state_version();
			let block: Block = generate_genesis_block(&spec, state_version)?;
			let raw_header = block.header().encode();
			let output_buf = if params.raw {
				raw_header
			} else {
				format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(Subcommand::ExportGenesisWasm(params)) => {
			let mut builder = sc_cli::LoggerBuilder::new("");
			builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
			let _ = builder.init();

			let raw_wasm_blob =
				extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
			let output_buf = if params.raw {
				raw_wasm_blob
			} else {
				format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
			};

			if let Some(output) = &params.output {
				std::fs::write(output, output_buf)?;
			} else {
				std::io::stdout().write_all(&output_buf)?;
			}

			Ok(())
		},
		Some(Subcommand::Benchmark(cmd)) =>
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;

				#[cfg(feature = "snowbridge-native")]
				if runner.config().chain_spec.is_snowbridge() {
					return runner.sync_run(|config| {
						cmd.run::<Block, crate::service::SnowbridgeRuntimeExecutor>(config)
					})
				}

				#[cfg(feature = "snowblink-native")]
				if runner.config().chain_spec.is_snowblink() {
					return runner.sync_run(|config| {
						cmd.run::<Block, crate::service::SnowblinkRuntimeExecutor>(config)
					})
				}

				#[cfg(feature = "snowbase-native")]
				if runner.config().chain_spec.is_snowbase() {
					return runner.sync_run(|config| {
						cmd.run::<Block, crate::service::SnowbaseRuntimeExecutor>(config)
					})
				}

				Err("Chain doesn't support benchmarking".into())
			} else {
				Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
					.into())
			},
		None => {
			let runner = cli.create_runner(&*cli.run)?;

			runner.run_node_until_exit(|config| async move {
				let para_id = Extensions::try_get(&*config.chain_spec).map(|e| e.para_id);

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name().to_string()]
						.iter()
						.chain(cli.relaychain_args.iter()),
				);

				let id = ParaId::from(cli.run.parachain_id.or(para_id).unwrap_or(100));

				let parachain_account =
					AccountIdConversion::<polkadot_primitives::v0::AccountId>::into_account(&id);

				let state_version =
					RelayChainCli::native_runtime_version(&config.chain_spec).state_version();
				let block: Block = generate_genesis_block(&config.chain_spec, state_version)
					.map_err(|e| format!("{:?}", e))?;
				let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Parachain id: {:?}", id);
				info!("Parachain Account: {}", parachain_account);
				info!("Parachain genesis state: {}", genesis_state);
				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				#[cfg(feature = "snowbridge-native")]
				if config.chain_spec.is_snowbridge() {
					return crate::service::start_parachain_node::<
						snowbridge_runtime::RuntimeApi,
						crate::service::SnowbridgeRuntimeExecutor,
					>(config, polkadot_config, id)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				}

				#[cfg(feature = "snowblink-native")]
				if config.chain_spec.is_snowblink() {
					return crate::service::start_parachain_node::<
						snowblink_runtime::RuntimeApi,
						crate::service::SnowblinkRuntimeExecutor,
					>(config, polkadot_config, id)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				}

				#[cfg(feature = "snowbase-native")]
				if config.chain_spec.is_snowbase() {
					return crate::service::start_parachain_node::<
						snowbase_runtime::RuntimeApi,
						crate::service::SnowbaseRuntimeExecutor,
					>(config, polkadot_config, id)
					.await
					.map(|r| r.0)
					.map_err(Into::into)
				}

				Err("unknown runtime".into())
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn sc_cli::ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
