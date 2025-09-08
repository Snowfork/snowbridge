use crate::{
	constants::*,
	parachains::{
		relaychain,
		relaychain::api::runtime_types::{
			pallet_xcm::pallet::Call as RelaychainPalletXcmCall,
			sp_weights::weight_v2::Weight as RelaychainWeight,
			staging_xcm::v3::multilocation::MultiLocation as RelaychainMultiLocation,
			westend_runtime::RuntimeCall as RelaychainRuntimeCall,
			xcm::{
				double_encoded::DoubleEncoded as RelaychainDoubleEncoded,
				v3::{
					junction::{
						Junction as RelaychainJunction,
						Junction::AccountId32 as RelaychainAccountId32,
						NetworkId as RelaychainNetworkId,
					},
					junctions::Junctions as RelaychainJunctions,
					multiasset::{
						AssetId as RelaychainAssetId, Fungibility as RelaychainFungibility,
						MultiAsset as RelaychainMultiAsset,
						MultiAssetFilter as RelaychainMultiAssetFilter,
						MultiAssets as RelaychainMultiAssets,
						WildMultiAsset as RelaychainWildMultiAsset,
					},
					Instruction as RelaychainInstruction, OriginKind as RelaychainOriginKind,
					WeightLimit as RelaychainWeightLimit, Xcm as RelaychainXcm,
				},
				VersionedLocation as RelaychainVersionedLocation,
				VersionedXcm as RelaychainVersionedXcm,
			},
		},
	},
};
use alloy::{
	dyn_abi::DynSolValue,
	eips::BlockNumberOrTag,
	network::TransactionBuilder,
	primitives::{Address, Bytes, FixedBytes, Log, B256, U256},
	providers::{DynProvider, Provider, ProviderBuilder, WsConnect},
	rpc::types::{Filter, TransactionRequest},
	signers::local::PrivateKeySigner,
	sol_types::SolEvent,
};
use futures::StreamExt;
use pair_signer::PairSigner;
use sp_core::{sr25519::Pair, Pair as PairT};
use subxt::{
	config::DefaultExtrinsicParams,
	events::StaticEvent,
	utils::{AccountId32, MultiAddress, H256},
	Config, OnlineClient, PolkadotConfig,
};

#[cfg(feature = "legacy-v1")]
use crate::contracts::i_gateway::IGateway;
#[cfg(not(feature = "legacy-v1"))]
use crate::contracts::i_gateway_v1::IGatewayV1 as IGateway;
#[cfg(not(feature = "legacy-v1"))]
use crate::contracts::i_gateway_v2::IGatewayV2;

/// Custom config that works with Statemint
pub enum AssetHubConfig {}

impl Config for AssetHubConfig {
	type AccountId = <PolkadotConfig as Config>::AccountId;
	type Address = <PolkadotConfig as Config>::Address;
	type Signature = <PolkadotConfig as Config>::Signature;
	type Hasher = <PolkadotConfig as Config>::Hasher;
	type Header = <PolkadotConfig as Config>::Header;
	type ExtrinsicParams = DefaultExtrinsicParams<AssetHubConfig>;
	type AssetId = <PolkadotConfig as Config>::AssetId;
}

/// A concrete PairSigner implementation which relies on `sr25519::Pair` for signing
/// and that PolkadotConfig is the runtime configuration.
pub mod pair_signer {
	use super::*;
	use sp_core::sr25519;
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		MultiSignature as SpMultiSignature,
	};
	use subxt::{
		config::substrate::{AccountId32, MultiSignature},
		tx::Signer,
	};

	/// A [`Signer`] implementation for [`polkadot_sdk::sp_core::sr25519::Pair`].
	#[derive(Clone)]
	pub struct PairSigner {
		account_id: <PolkadotConfig as Config>::AccountId,
		signer: sr25519::Pair,
	}

	impl PairSigner {
		/// Creates a new [`Signer`] from an [`sp_core::sr25519::Pair`].
		pub fn new(signer: sr25519::Pair) -> Self {
			let account_id =
				<SpMultiSignature as Verify>::Signer::from(signer.public()).into_account();
			Self {
				// Convert `sp_core::AccountId32` to `subxt::config::substrate::AccountId32`.
				//
				// This is necessary because we use `subxt::config::substrate::AccountId32` and no
				// From/Into impls are provided between `sp_core::AccountId32` because
				// `polkadot-sdk` isn't a direct dependency in subxt.
				//
				// This can also be done by provided a wrapper type around
				// `subxt::config::substrate::AccountId32` to implement such conversions but
				// that also most likely requires a custom `Config` with a separate `AccountId` type
				// to work properly without additional hacks.
				account_id: AccountId32(account_id.into()),
				signer,
			}
		}
	}

	impl Signer<PolkadotConfig> for PairSigner {
		fn account_id(&self) -> <PolkadotConfig as Config>::AccountId {
			self.account_id.clone()
		}

		fn sign(&self, signer_payload: &[u8]) -> <PolkadotConfig as Config>::Signature {
			let signature = self.signer.sign(signer_payload);
			MultiSignature::Sr25519(signature.0)
		}
	}
}

pub struct TestClients {
	pub asset_hub_client: Box<OnlineClient<AssetHubConfig>>,
	pub bridge_hub_client: Box<OnlineClient<PolkadotConfig>>,
	pub relaychain_client: Box<OnlineClient<PolkadotConfig>>,
	pub ethereum_client: Box<DynProvider>,
}

pub async fn initial_clients() -> Result<TestClients, Box<dyn std::error::Error>> {
	let bridge_hub_client: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*BRIDGE_HUB_WS_URL).to_string())
			.await
			.expect("can not connect to bridgehub");

	let asset_hub_client: OnlineClient<AssetHubConfig> =
		OnlineClient::from_url((*ASSET_HUB_WS_URL).to_string())
			.await
			.expect("can not connect to assethub");

	let relaychain_client: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*RELAY_CHAIN_WS_URL).to_string())
			.await
			.expect("can not connect to relaychain");

	// Initialize a signer with a private key
	let signer: PrivateKeySigner = (*ETHEREUM_KEY).to_string().parse()?;

	// Create the provider.
	let ws = WsConnect::new((*ETHEREUM_API).to_string());

	let ethereum_provider = ProviderBuilder::new().wallet(signer).connect_ws(ws).await?.erased();

	Ok(TestClients {
		asset_hub_client: Box::new(asset_hub_client),
		bridge_hub_client: Box::new(bridge_hub_client),
		relaychain_client: Box::new(relaychain_client),
		ethereum_client: Box::new(ethereum_provider),
	})
}

pub async fn wait_for_bridgehub_event<Ev: StaticEvent>(
	bridge_hub_client: &Box<OnlineClient<PolkadotConfig>>,
) {
	let mut blocks = bridge_hub_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(500);

	let mut substrate_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling bridgehub block {} for expected event.", block.number());
		let events = block.events().await.expect("read block events");
		for event in events.find::<Ev>() {
			let _ = event.expect("expect upgrade");
			println!("Event found at bridgehub block {}.", block.number());
			substrate_event_found = true;
			break
		}
		if substrate_event_found {
			break
		}
	}
	assert!(substrate_event_found);
}

pub async fn wait_for_assethub_event<Ev: StaticEvent>(
	asset_hub_client: &Box<OnlineClient<AssetHubConfig>>,
) {
	let mut blocks = asset_hub_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("block subscription")
		.take(5);

	let mut substrate_event_found = false;
	while let Some(Ok(block)) = blocks.next().await {
		println!("Polling assethub block {} for expected event.", block.number());
		let events = block.events().await.expect("read block events");
		for event in events.find::<Ev>() {
			let _ = event.expect("expect upgrade");
			println!(
				"Event found at assethub block {}: {}::{}",
				block.number(),
				<Ev as StaticEvent>::PALLET,
				<Ev as StaticEvent>::EVENT,
			);
			substrate_event_found = true;
			break
		}
		if substrate_event_found {
			break
		}
	}
	assert!(substrate_event_found);
}

pub async fn wait_for_ethereum_event<Ev: SolEvent>(
	ethereum_client: Box<dyn Provider>,
	contract_address: Address,
) {
	let filter = Filter::new()
		// By NOT specifying an `event` or `event_signature` we listen to ALL events of the
		// contract.
		.address(contract_address)
		.from_block(BlockNumberOrTag::Latest);

	let logs = ethereum_client.subscribe_logs(&filter).await.expect("logs");
	let mut stream = logs.into_stream();

	let mut ethereum_event_found = false;
	let expected_topic0: B256 = Ev::SIGNATURE_HASH.into();
	while let Some(log) = stream.next().await {
		match log.topic0() {
			Some(&topic0) =>
				if topic0 == expected_topic0 {
					println!("Event found at ethereum block {:?}", log.block_number);
					ethereum_event_found = true;
					break
				},
			_ => (),
		}
		if ethereum_event_found {
			break
		}
	}
	assert!(ethereum_event_found);
}

pub struct SudoResult {
	pub block_hash: H256,
	pub extrinsic_hash: H256,
}

pub async fn get_balance(
	client: Box<DynProvider>,
	who: Address,
) -> Result<U256, Box<dyn std::error::Error>> {
	let balance = client.get_balance(who).await?;
	Ok(balance)
}

pub async fn fund_account(
	client: Box<dyn Provider>,
	address_to: Address,
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let tx = TransactionRequest::default()
		.to(address_to)
		.with_gas_price(GAS_PRICE)
		.value(U256::from(amount));
	let pending_tx = client.send_transaction(tx).await?;
	println!("Pending transaction... {}", pending_tx.tx_hash());

	// Wait for the transaction to be included and get the receipt
	let receipt = pending_tx.get_receipt().await?;
	println!(
		"Transaction included in block {}",
		receipt.block_number.expect("Failed to get block number")
	);
	Ok(())
}

pub async fn governance_bridgehub_call_from_relay_chain(
	call: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(BRIDGE_HUB_PARA_ID)),
	}));
	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::UnpaidExecution {
			weight_limit: RelaychainWeightLimit::Unlimited,
			check_origin: None,
		},
		RelaychainInstruction::Transact {
			origin_kind: RelaychainOriginKind::Superuser,
			require_weight_at_most: RelaychainWeight { ref_time: weight, proof_size },
			call: RelaychainDoubleEncoded { encoded: call },
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn snowbridge_assethub_call_from_relay_chain(
	call: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(ASSET_HUB_PARA_ID)),
	}));

	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::UnpaidExecution {
			weight_limit: RelaychainWeightLimit::Unlimited,
			check_origin: None,
		},
		RelaychainInstruction::DescendOrigin(RelaychainJunctions::X1(
			RelaychainJunction::Parachain(BRIDGE_HUB_PARA_ID),
		)),
		RelaychainInstruction::DescendOrigin(RelaychainJunctions::X1(
			RelaychainJunction::PalletInstance(INBOUND_QUEUE_PALLET_INDEX_V2),
		)),
		RelaychainInstruction::UniversalOrigin(RelaychainJunction::GlobalConsensus(
			RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
		)),
		RelaychainInstruction::Transact {
			origin_kind: RelaychainOriginKind::SovereignAccount,
			require_weight_at_most: RelaychainWeight { ref_time: weight, proof_size },
			call: RelaychainDoubleEncoded { encoded: call },
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn assethub_deposit_eth_on_penpal_call_from_relay_chain(
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(ASSET_HUB_PARA_ID)),
	}));

	let dot_location = RelaychainMultiLocation { parents: 1, interior: RelaychainJunctions::Here };
	let eth_location = RelaychainMultiLocation {
		parents: 2,
		interior: RelaychainJunctions::X1(RelaychainJunction::GlobalConsensus(
			RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
		)),
	};

	let eth_asset: RelaychainMultiAsset = RelaychainMultiAsset {
		id: RelaychainAssetId::Concrete(eth_location),
		fun: RelaychainFungibility::Fungible(3_000_000_000_000u128),
	};
	let dot_asset: RelaychainMultiAsset = RelaychainMultiAsset {
		id: RelaychainAssetId::Concrete(dot_location),
		fun: RelaychainFungibility::Fungible(3_000_000_000_000u128),
	};

	let account_location: RelaychainMultiLocation = RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainAccountId32 {
			network: None,
			id: (*FERDIE_PUBLIC).into(),
		}),
	};

	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::UnpaidExecution {
			weight_limit: RelaychainWeightLimit::Unlimited,
			check_origin: None,
		},
		RelaychainInstruction::DescendOrigin(RelaychainJunctions::X1(
			RelaychainJunction::Parachain(BRIDGE_HUB_PARA_ID),
		)),
		RelaychainInstruction::DescendOrigin(RelaychainJunctions::X1(
			RelaychainJunction::PalletInstance(INBOUND_QUEUE_PALLET_INDEX_V2),
		)),
		RelaychainInstruction::UniversalOrigin(RelaychainJunction::GlobalConsensus(
			RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
		)),
		RelaychainInstruction::WithdrawAsset(RelaychainMultiAssets(vec![RelaychainMultiAsset {
			id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
				parents: 1,
				interior: RelaychainJunctions::Here,
			}),
			fun: RelaychainFungibility::Fungible(3_000_000_000_000_u128),
		}])),
		RelaychainInstruction::ReserveAssetDeposited(RelaychainMultiAssets(vec![
			RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 2,
					interior: RelaychainJunctions::X1(RelaychainJunction::GlobalConsensus(
						RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
					)),
				}),
				fun: RelaychainFungibility::Fungible(3_000_000_000_000_u128),
			},
		])),
		RelaychainInstruction::DepositReserveAsset {
			// Send the token plus some eth for execution fees
			assets: RelaychainMultiAssetFilter::Definite(RelaychainMultiAssets(vec![
				dot_asset, eth_asset,
			])),
			// Penpal
			dest: RelaychainMultiLocation {
				parents: 1,
				interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(PENPAL_PARA_ID)),
			},
			xcm: RelaychainXcm(vec![
				// Pay fees on Penpal.
				RelaychainInstruction::BuyExecution {
					fees: RelaychainMultiAsset {
						id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
							parents: 1,
							interior: RelaychainJunctions::Here,
						}),
						fun: RelaychainFungibility::Fungible(2_000_000_000_000_u128),
					},
					weight_limit: RelaychainWeightLimit::Limited(RelaychainWeight {
						ref_time: weight,
						proof_size,
					}),
				},
				// Deposit assets to beneficiary.
				RelaychainInstruction::DepositAsset {
					assets: RelaychainMultiAssetFilter::Wild(RelaychainWildMultiAsset::AllCounted(
						2,
					)),
					beneficiary: account_location,
				},
			]),
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn governance_assethub_call_from_relay_chain_sudo_as(
	who: MultiAddress<AccountId32, ()>,
	call: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(ASSET_HUB_PARA_ID)),
	}));

	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::BuyExecution {
			fees: RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 0,
					interior: RelaychainJunctions::Here,
				}),
				fun: RelaychainFungibility::Fungible(7_000_000_000_000_u128),
			},
			weight_limit: RelaychainWeightLimit::Limited(RelaychainWeight {
				ref_time: weight,
				proof_size,
			}),
		},
		RelaychainInstruction::Transact {
			origin_kind: RelaychainOriginKind::Superuser,
			require_weight_at_most: RelaychainWeight { ref_time: weight, proof_size },
			call: RelaychainDoubleEncoded { encoded: call },
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api.sudo_as(
		who,
		RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }),
	);

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn fund_agent(
	agent_id: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, *(test_clients.ethereum_client.clone()));
	let agent_address = gateway.agentOf(FixedBytes::from(agent_id)).call().await?;

	println!("agent address {}", hex::encode(agent_address));

	fund_account(test_clients.ethereum_client, agent_address, amount)
		.await
		.expect("fund account");
	Ok(())
}

pub fn print_event_log_for_unit_tests(log: &Log) {
	let topics: Vec<String> = log.topics().iter().map(|t| hex::encode(t.0.as_slice())).collect();
	println!("Log {{");
	println!("	address: hex!(\"{}\").into(),", hex::encode(log.address));
	println!("	topics: vec![");
	for topic in topics.iter() {
		println!("		hex!(\"{}\").into(),", topic);
	}
	println!("	],");
	println!("	data: hex!(\"{}\").into(),", hex::encode(&log.data.data));

	println!("}}")
}

pub async fn governance_assethub_call_from_relay_chain(
	call: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(ASSET_HUB_PARA_ID)),
	}));
	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::UnpaidExecution {
			weight_limit: RelaychainWeightLimit::Unlimited,
			check_origin: None,
		},
		RelaychainInstruction::Transact {
			origin_kind: RelaychainOriginKind::Superuser,
			require_weight_at_most: RelaychainWeight { ref_time: weight, proof_size },
			call: RelaychainDoubleEncoded { encoded: call },
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub async fn deposit_eth_to_penpal(
	account: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");

	let sudo = Pair::from_string("//Alice", None).expect("cannot create sudo keypair");

	let signer: PairSigner = PairSigner::new(sudo);

	let weight = 180000000000;
	let proof_size = 900000;

	let account_location: RelaychainMultiLocation = RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainAccountId32 {
			network: None,
			id: account.into(),
		}),
	};
	let dest = Box::new(RelaychainVersionedLocation::V3(RelaychainMultiLocation {
		parents: 0,
		interior: RelaychainJunctions::X1(RelaychainJunction::Parachain(PENPAL_PARA_ID)),
	}));

	let message = Box::new(RelaychainVersionedXcm::V3(RelaychainXcm(vec![
		RelaychainInstruction::BuyExecution {
			fees: RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 0,
					interior: RelaychainJunctions::Here,
				}),
				fun: RelaychainFungibility::Fungible(amount),
			},
			weight_limit: RelaychainWeightLimit::Limited(RelaychainWeight {
				ref_time: weight,
				proof_size,
			}),
		},
		RelaychainInstruction::ReserveAssetDeposited(RelaychainMultiAssets(vec![
			RelaychainMultiAsset {
				id: RelaychainAssetId::Concrete(RelaychainMultiLocation {
					parents: 2,
					interior: RelaychainJunctions::X1(RelaychainJunction::GlobalConsensus(
						RelaychainNetworkId::Ethereum { chain_id: ETHEREUM_CHAIN_ID },
					)),
				}),
				fun: RelaychainFungibility::Fungible(amount),
			},
		])),
		RelaychainInstruction::DepositAsset {
			assets: RelaychainMultiAssetFilter::Wild(RelaychainWildMultiAsset::AllCounted(2)),
			beneficiary: account_location,
		},
	])));

	let sudo_api = relaychain::api::sudo::calls::TransactionApi;
	let sudo_call = sudo_api
		.sudo(RelaychainRuntimeCall::XcmPallet(RelaychainPalletXcmCall::send { dest, message }));

	let result = test_clients
		.relaychain_client
		.tx()
		.sign_and_submit_then_watch_default(&sudo_call, &signer)
		.await
		.expect("send through sudo call.")
		.wait_for_finalized_success()
		.await
		.expect("sudo call success");

	println!("Sudo call issued at relaychain block hash {:?}", result.extrinsic_hash());

	Ok(())
}

pub fn build_native_asset(token: Address, amount: u128) -> Bytes {
	let kind_token = DynSolValue::Uint(U256::from(0u8), 256);
	let token_token = DynSolValue::Address(token);
	let amount_token = DynSolValue::Uint(U256::from(amount), 256);
	Bytes::from(DynSolValue::Tuple(vec![kind_token, token_token, amount_token]).abi_encode())
}

pub async fn fund_agent_v2(
	agent_id: [u8; 32],
	amount: u128,
) -> Result<(), Box<dyn std::error::Error>> {
	let test_clients = initial_clients().await.expect("initialize clients");
	let agent_address = get_agent_address(test_clients.ethereum_client.clone(), agent_id)
		.await
		.expect("get agent address");

	fund_account(test_clients.ethereum_client, agent_address, amount)
		.await
		.expect("fund account");
	Ok(())
}

pub async fn get_agent_address(
	ethereum_client: Box<DynProvider>,
	agent_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGatewayV2::new(gateway_addr, *ethereum_client);
	let agent_address = gateway.agentOf(FixedBytes::from(agent_id)).call().await?;
	Ok(agent_address)
}

pub async fn get_token_address(
	ethereum_client: Box<DynProvider>,
	token_id: [u8; 32],
) -> Result<Address, Box<dyn std::error::Error>> {
	let gateway_addr: Address = (*GATEWAY_PROXY_CONTRACT).into();
	let gateway = IGateway::new(gateway_addr, *ethereum_client);
	let token_address = gateway.tokenAddressOf(FixedBytes::from(token_id)).call().await?;
	Ok(token_address)
}
