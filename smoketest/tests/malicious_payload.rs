use std::sync::Arc;

use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{DynProvider, Provider, ProviderBuilder, WsConnect};
use alloy::signers::local::PrivateKeySigner;
use codec::{DecodeAll, Encode};
use futures::StreamExt;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::ws_client::WsClientBuilder;
use serde_json::Value;

use alloy::primitives::FixedBytes;
use snowbridge_smoketest::{
	constants::*,
	contracts::beefy_client::{
		BeefyClient, BeefyClient::Commitment, BeefyClient::MMRLeaf, BeefyClient::PayloadItem,
		BeefyClient::ValidatorProof,
	},
	parachains::relaychain::{
		self,
		api::{
			beefy_mmr_leaf::storage::types::beefy_authorities,
			runtime_apis::beefy_api,
			runtime_types::{
				sp_consensus_beefy::{
					commitment::Commitment as spCommitment,
					ecdsa_crypto::{Public, Signature},
					payload::Payload,
					ForkVotingProof, FutureBlockVotingProof, VoteMessage,
				},
				sp_mmr_primitives::AncestryProof,
				sp_runtime::generic::{digest::Digest, header::Header},
				sp_session::MembershipProof,
			},
		},
	},
};
use sp_mmr_primitives::AncestryProof as MmrAncestryProof;
use subxt_signer::sr25519::dev;

use sp_consensus_beefy;
// use sp_consensus_beefy::ecdsa_crypto::Pair as PairT3;
use sp_core::{ecdsa::Pair, hex2array, hexdisplay::AsBytesRef, Pair as PairT};
use sp_crypto_hashing::keccak_256;
use subxt::{
	backend::rpc::RpcClient,
	config::substrate::{BlakeTwo256, SubstrateHeader},
	OnlineClient, PolkadotConfig,
};

fn create_ticket_id(account: Address, commitment_hash: [u8; 32]) -> [u8; 32] {
	let mut buf = [0u8; 64];
	// account is 20 bytes -> left-pad to 32 bytes
	buf[12..32].copy_from_slice(account.as_slice());
	buf[32..64].copy_from_slice(&commitment_hash);
	let hash = keccak_256(&buf);
	let mut ticket_id = [0u8; 32];
	ticket_id.copy_from_slice(&hash);
	ticket_id
}
// const GATEWAY_V2_ADDRESS: [u8; 20] = hex!("ee9170abfbf9421ad6dd07f6bdec9d89f2b581e0");

pub struct TestClients {
	pub relaychain_client: Box<OnlineClient<PolkadotConfig>>,
	pub ethereum_client: Box<DynProvider>,
	pub beefy_client: Box<BeefyClient::BeefyClientInstance<Arc<DynProvider>>>,
}

async fn initialize_clients() -> Result<TestClients, Box<dyn std::error::Error>> {
	// Initialize a signer with a private key
	let signer: PrivateKeySigner = (*ETHEREUM_KEY).to_string().parse()?;

	// Create the provider.
	let ws = WsConnect::new((*ETHEREUM_API).to_string());

	let ethereum_provider = ProviderBuilder::new().connect_ws(ws.clone()).await?.erased();
	let ethereum_signed_provider =
		ProviderBuilder::new().wallet(signer).connect_ws(ws).await?.erased();
	let relaychain_rpc_client = RpcClient::from_url((*RELAY_CHAIN_WS_URL).to_string())
		.await
		.expect("can not connect to relaychain RPC");
	let relaychain_client = OnlineClient::from_rpc_client(relaychain_rpc_client).await?;
	let beefy_client_addr: Address = BEEFY_CLIENT_CONTRACT.into();
	let beefy_client = BeefyClient::new(beefy_client_addr, Arc::new(ethereum_signed_provider));
	Ok(TestClients {
		relaychain_client: Box::new(relaychain_client),
		ethereum_client: Box::new(ethereum_provider),
		beefy_client: Box::new(beefy_client),
	})
}

fn validator_proof(
	signature_bytes: &[u8],
	signer_index: usize,
	validator_secp256k1_bytes: &Vec<[u8; 20]>,
	validator_proofs: &[[FixedBytes<32>; 2]; 4],
) -> ValidatorProof {
	let mut r = FixedBytes([0u8; 32]);
	let mut s = FixedBytes([0u8; 32]);
	r.copy_from_slice(&signature_bytes[0..32]);
	s.copy_from_slice(&signature_bytes[32..64]);

	// For legacy format, convert 0/1 to 27/28
	let v_raw = signature_bytes[64];
	let v = match v_raw {
		0 => 27,
		1 => 28,
		_ => panic!("v can only be 0 or 1"),
	};

	println!("r: {:?}, s: {:?}, v: {:?}", r, s, v);

	ValidatorProof {
		v,
		r,
		s,
		index: U256::from(signer_index),
		account: Address::from_slice(&validator_secp256k1_bytes[signer_index]),
		proof: validator_proofs[signer_index].to_vec(),
	}
}

// TODO: reuse from polkadot-sdk
#[derive(Clone, Debug)]
enum EquivocationType {
	FutureBlockEquivocation,
	ForkEquivocation,
}

#[derive(Clone)]
struct TestConfig {
	submit_initial: bool,
	submit_final: bool,
	report_equivocation: bool,
}

#[tokio::test]
async fn malicious_payload() {
	// Setup clients
	// ---
	let test_clients = initialize_clients().await.expect("initialize clients");

	let mut blocks_sub = test_clients
		.relaychain_client
		.blocks()
		.subscribe_finalized()
		.await
		.expect("subscribe to blocks");

	let test_config =
		TestConfig { submit_initial: true, submit_final: true, report_equivocation: true };

	let equivocation_variants =
		vec![EquivocationType::ForkEquivocation, EquivocationType::FutureBlockEquivocation];
	for equivocation_variant in equivocation_variants.clone() {
		println!("Testing malicious payload with equivocation variant: {:?}", equivocation_variant);
		malicious_payload_inner(
			equivocation_variant,
			test_config.clone(),
			&test_clients,
			&mut blocks_sub,
		)
		.await;
	}

	let mut slashed_event_count = 0;
	// Watch blocks until offence & slash are reported
	while let Some(block) = blocks_sub.next().await {
		let block = block.expect("get block");
		let block_number = block.header().number;
		let block_hash = block.hash();

		if block_number % 5 == 0 {
			println!("Processing block #{} (Hash: {})", block_number, block_hash);
		}

		let events = block.events().await.expect("get events");

		let slashed_events =
			events.find::<relaychain::api::staking::events::Slashed>().collect::<Vec<_>>();
		if slashed_events.len() > 0 {
			println!("Slashed events found in block #{}: {:?}", block_number, slashed_events);
			slashed_event_count += slashed_events.len();
		}
		if slashed_event_count >= equivocation_variants.len() {
			println!("Slashed event count reached: {}", slashed_event_count);
			break;
		}
	}
}

async fn malicious_payload_inner(
	equivocation_type: EquivocationType,
	test_config: TestConfig,
	test_clients: &TestClients,
	blocks_sub: &mut subxt::backend::StreamOf<
		Result<subxt::blocks::Block<PolkadotConfig, OnlineClient<PolkadotConfig>>, subxt::Error>,
	>,
) {
	let block_number = test_clients
		.beefy_client
		.latestBeefyBlock()
		.call()
		.await
		.expect("beefy client initialized");

	println!("block_number: {:?}", block_number);

	let call = test_clients.beefy_client.latestMMRRoot();
	let current_mmr_root = call.call().await.expect("commit valid");
	println!("current mmr root: {:?}", current_mmr_root);
	let payload_data = FixedBytes([0; 32]);

	if current_mmr_root == *payload_data {
		println!("NOTE: BEEFY client already has malicious mmr payload");
	}

	// let randao_delay = test_clients
	// 	.beefy_client
	// 	.randao_commit_delay()
	// 	.call()
	// 	.await
	// 	.expect("beefy client initialized");
	// println!("randao delay: {:?}", randao_delay);

	let payload =
		vec![PayloadItem { payloadID: FixedBytes([109, 104]), data: Bytes::from(payload_data) }];
	let equivocation_block = match equivocation_type {
		EquivocationType::ForkEquivocation => (block_number as u32) + 1,
		EquivocationType::FutureBlockEquivocation => (block_number as u32) + 1000,
	};
	let current_validator_set = test_clients
		.beefy_client
		.currentValidatorSet()
		.call()
		.await
		.expect("beefy client initialized");

	let commitment = Commitment {
		payload: payload.clone(),
		blockNumber: equivocation_block,
		validatorSetID: (current_validator_set.id as u64),
	};

	let malicious_suris = vec!["//Westend04", "//Westend01", "//Westend02"];

	let malicious_authorities =
		malicious_suris.iter().map(|suri| Pair::from_string(suri, None).unwrap());
	println!(
		"malicious_authorities: {:?}",
		malicious_authorities.clone().map(|auth| auth.public()).collect::<Vec<_>>()
	);

	let sp_payload = sp_consensus_beefy::Payload::from_single_entry(
		*payload[0].payloadID,
		payload[0].data.to_vec(),
	);
	let sp_commitment = sp_consensus_beefy::Commitment {
		payload: sp_payload,
		block_number: commitment.blockNumber,
		validator_set_id: commitment.validatorSetID,
	};
	println!("sp_commitment: {:?}", sp_commitment);
	let encoded_commitment = sp_commitment.encode();
	println!("encoded commitment: {:?}", encoded_commitment);
	let hashed_commitment = &keccak_256(encoded_commitment.as_bytes_ref());
	println!("hashed commitment: {:?}", hashed_commitment);
	println!("hashed commitment hex: {:?}", hex::encode(hashed_commitment));

	let malicious_signatures = malicious_authorities
		.map(|pair| pair.sign_prehashed(hashed_commitment))
		.collect::<Vec<_>>();

	println!(
		"malicious_signatures: {:?}, raw: {:?}",
		malicious_signatures,
		malicious_signatures.iter().map(|sig| sig.0).collect::<Vec<_>>()
	);

	let bitfield: Vec<U256> = vec![U256::from(0b0111)];

	let (validator_secp256k1_bytes, validator_set_root, validator_proofs) = {
		let validator_secp256k1_bytes = vec![
			hex2array!("fd4de54fb46fb25358323c12484dea951da5db48"),
			hex2array!("96fade2050ee5b75c01964e556b49a7c53de0bc5"),
			hex2array!("054426fc7aab50156c0dfcdcd607e7045cc58d9e"),
			hex2array!("a601c19ad010f21031f7317f08b4f0046db6ce2a"),
		];

		let keccak_validator_secp256k1_bytes: Vec<FixedBytes<32>> = validator_secp256k1_bytes
			.iter()
			.map(|key| FixedBytes(keccak_256(key)))
			.collect();

		let keccak01 = FixedBytes(keccak_256(
			&[keccak_validator_secp256k1_bytes[0], keccak_validator_secp256k1_bytes[1]].concat(),
		));
		let keccak23 = FixedBytes(keccak_256(
			&[keccak_validator_secp256k1_bytes[2], keccak_validator_secp256k1_bytes[3]].concat(),
		));
		let validator_set_root = keccak_256(&[keccak01, keccak23].concat());

		let validator_proofs = [
			[keccak_validator_secp256k1_bytes[1], keccak23],
			[keccak_validator_secp256k1_bytes[0], keccak23],
			[keccak_validator_secp256k1_bytes[3], keccak01],
			[keccak_validator_secp256k1_bytes[2], keccak01],
		];
		(validator_secp256k1_bytes, validator_set_root, validator_proofs)
	};

	println!("validator proofs: {:?}", validator_proofs);

	let signer_index = match equivocation_type {
		EquivocationType::ForkEquivocation => 0, // Fork equivocation uses first signer
		EquivocationType::FutureBlockEquivocation => 2, // Future block equivocation uses second signer
	};
	let malicious_authority: Pair = Pair::from_string(malicious_suris[signer_index], None).unwrap();

	let proof = validator_proof(
		malicious_signatures[signer_index].0.as_slice(),
		signer_index,
		&validator_secp256k1_bytes,
		&validator_proofs,
	);

	let suggested_gas_price = test_clients.ethereum_client.get_gas_price().await;
	let higher_gas_price = suggested_gas_price.unwrap() * 3; // 200% higher
	if test_config.submit_initial {
		let commit_delay = test_clients
			.beefy_client
			.randaoCommitDelay()
			.call()
			.await
			.expect("get randao commit delay");
		println!("RANDAO commit delay: {:?}", commit_delay);

		let call = test_clients
			.beefy_client
			.submitInitial(commitment.clone(), bitfield.clone(), proof);
		let result = call.gas_price(higher_gas_price).send().await;
		if result.is_ok() {
			println!("successful submitInit: {:?}", result.as_ref().unwrap());
		} else {
			println!(
				"{:?}",
				result.as_ref().err().unwrap()
			);
		}
		assert!(result.is_ok());
		println!("waiting for receipt");
		let receipt = result.unwrap().get_receipt().await.expect("get receipt");
		println!("receipt (submitInitial): {:?}", receipt);
		let mut stream = test_clients
			.ethereum_client
			.subscribe_blocks()
			.await
			.unwrap()
			.into_result_stream()
			.take(commit_delay.try_into().expect("commit_delay fits usize"));

		while let Some(_block) = stream.next().await {}

		let call = test_clients.beefy_client.commitPrevRandao(FixedBytes(*hashed_commitment));
		let result_raw = call.gas_price(higher_gas_price).send().await;
		let result = result_raw.expect("commit valid");
		println!("result (commitPrevRandao): {:?}", result);
		let receipt = result.get_receipt().await.expect("get receipt");
		println!("receipt (commitPrevRandao): {:?}", receipt);
	}

	if test_config.submit_final {
		let mut stream = test_clients
			.ethereum_client
			.subscribe_blocks()
			.await
			.unwrap()
			.into_result_stream()
			.take(5);
		while let Some(_block) = stream.next().await {}

		let signer: PrivateKeySigner = (*ETHEREUM_KEY).to_string().parse().unwrap();
		let address = signer.address();
		let ticket_id = create_ticket_id(address, *hashed_commitment);
		let ticket = test_clients.beefy_client.tickets(FixedBytes(ticket_id)).call().await.unwrap();
		println!("ticket: {:?}", ticket);

		let call = test_clients
			.beefy_client
			.createFinalBitfield(FixedBytes(*hashed_commitment), bitfield);
		let bitfield = call.call().await.expect("commit valid");
		// println!("final bitfield: {:?}", result);

		assert_eq!(bitfield.len(), 1);
		let chosen_malicious_proofs = malicious_signatures
			.iter()
			.enumerate()
			.filter(|(i, _)| bitfield[0].bit(*i))
			.map(|(i, sig)| {
				//TODO: deduplicate with init sig
				let sig_bytes = sig.0.as_slice();

				validator_proof(sig_bytes, i, &validator_secp256k1_bytes, &validator_proofs)
			})
			.collect::<Vec<_>>();

		let mmr_leaf = MMRLeaf {
			version: 0,
			parentNumber: 0,
			parentHash: FixedBytes([0; 32]),
			nextAuthoritySetID: (current_validator_set.id as u64) + 1,
			nextAuthoritySetLen: 4,
			nextAuthoritySetRoot: FixedBytes(validator_set_root),
			parachainHeadsRoot: FixedBytes([0; 32]),
		};

		let call = test_clients.beefy_client.submitFinal(
			commitment,
			bitfield,
			chosen_malicious_proofs,
			mmr_leaf,
			vec![],
			U256::from(0),
		);

		let result = call.gas_price(higher_gas_price).send().await;
		if !result.is_ok() {
			println!("result (submitFinal): {:?}", result)
		}
		assert!(result.is_ok());
	}

	if test_config.report_equivocation {
		// let beefy_storage_api = relaychain::api::beefy::storage::StorageApi;
		// let validator_set_id = beefy_storage_api.validator_set_id();
		// println!("validator_set_id: {:?}", validator_set_id);

		let key_ownership_proof = {
			let proof_query = beefy_api::BeefyApi::generate_key_ownership_proof(
				&beefy_api::BeefyApi,
				// validator_set_id,
				current_validator_set.id as u64,
				// 0,
				Public { 0: malicious_authority.public().0 },
			);

			let key_ownership_proof_bytes = test_clients
				.relaychain_client
				.runtime_api()
				.at_latest()
				.await
				.expect("can not connect to relaychain")
				.call(proof_query)
				.await
				.expect("runtime query failed")
				.expect("validator set is not Some");
			// println!("{:?}", key_ownership_proof_bytes.0);

			MembershipProof::decode_all(&mut key_ownership_proof_bytes.0.as_slice()).unwrap()
		};
		// println!("{:?}", key_ownership_proof);

		// Create equivocation proof
		// ---
		match equivocation_type {
			EquivocationType::ForkEquivocation => {
				let client = WsClientBuilder::default()
					.build((*RELAY_CHAIN_WS_URL).to_string())
					.await
					.expect("connect to relaychain node rpc");

				let header = {
					let block_hash: Value = client
						.request::<Value, Vec<Value>>("chain_getBlockHash", vec![])
						.await
						.expect("get block hash");
					let header_substrate: SubstrateHeader<u32, BlakeTwo256> = client
						.request("chain_getHeader", vec![block_hash])
						.await
						.expect("get block header");
					Header {
						digest: Digest::decode_all(
							&mut header_substrate.digest.encode().as_bytes_ref(),
						)
						.expect("decode digest"),
						number: header_substrate.number,
						parent_hash: header_substrate.parent_hash,
						extrinsics_root: header_substrate.extrinsics_root,
						state_root: header_substrate.state_root,
					}
				};

				// TODO: use block_hash from prior to ensure same call
				let ancestry_proof = {
					let ancestry_proof_substrate: MmrAncestryProof<subxt::utils::H256> = client
						.request("mmr_generateAncestryProof", vec![equivocation_block])
						.await
						.expect("get ancestry proof");

					AncestryProof {
						prev_peaks: ancestry_proof_substrate.prev_peaks,
						prev_leaf_count: ancestry_proof_substrate.prev_leaf_count,
						leaf_count: ancestry_proof_substrate.leaf_count,
						items: ancestry_proof_substrate.items,
					}
				};

				println!("Ancestry Proof: {:?}", ancestry_proof);

				let equivocation_proof = ForkVotingProof {
					ancestry_proof,
					header,
					vote: VoteMessage {
						commitment: spCommitment {
							block_number: sp_commitment.block_number,
							payload: Payload(vec![(
								*payload[0].payloadID,
								payload[0].data.to_vec(),
							)]),
							validator_set_id: sp_commitment.validator_set_id,
						},
						id: Public { 0: malicious_authority.public().0 },
						signature: Signature(malicious_signatures[signer_index].0),
					},
				};

				let report = relaychain::api::tx()
					.beefy()
					.report_fork_voting(equivocation_proof, key_ownership_proof);

				let tx = test_clients
					.relaychain_client
					.tx()
					.sign_and_submit_default(&report, &dev::bob())
					.await
					.expect("submit report");

				println!("report_fork_equivocation transaction: {:?}", tx);
			},
			EquivocationType::FutureBlockEquivocation => {
				let equivocation_proof = FutureBlockVotingProof {
					vote: VoteMessage {
						commitment: spCommitment {
							block_number: sp_commitment.block_number,
							payload: Payload(vec![(
								*payload[0].payloadID,
								payload[0].data.to_vec(),
							)]),
							validator_set_id: sp_commitment.validator_set_id,
						},
						id: Public { 0: malicious_authority.public().0 },
						signature: Signature(malicious_signatures[signer_index].0),
					},
				};

				let report = relaychain::api::tx()
					.beefy()
					.report_future_block_voting(equivocation_proof, key_ownership_proof);

				let tx = test_clients
					.relaychain_client
					.tx()
					.sign_and_submit_default(&report, &dev::alice())
					.await
					.expect("submit report");

				println!("report_future_block_equivocation transaction: {:?}", tx);
			},
		};
	}

	// Watch blocks until offence & slash are reported
	while let Some(block) = blocks_sub.next().await {
		let block = block.expect("get block");
		let block_number = block.header().number;
		let block_hash = block.hash();

		if block_number % 5 == 0 {
			println!("Processing block #{} (Hash: {})", block_number, block_hash);
		}

		let events = block.events().await.expect("get events");

		let offence_events =
			events.find::<relaychain::api::offences::events::Offence>().collect::<Vec<_>>();
		if offence_events.len() > 0 {
			println!("Offence events found in block #{}: {:?}", block_number, offence_events);
		}

		let slash_report_events = events
			.find::<relaychain::api::staking::events::SlashReported>()
			.collect::<Vec<_>>();
		if slash_report_events.len() > 0 {
			println!(
				"Slash reported events found in block #{}: {:?}",
				block_number, slash_report_events
			);
			return ();
		}
	}
}
