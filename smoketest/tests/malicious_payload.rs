use std::{sync::Arc, time::Duration};

use codec::{DecodeAll, Encode};
use ethers::{
	abi::Address,
	core::{k256::ecdsa::SigningKey, types::U256},
	prelude::*,
};

use snowbridge_smoketest::{
	constants::*,
	contracts::{
		beefy_client::{BeefyClient, Commitment, PayloadItem},
		shared_types::{Mmrleaf, ValidatorProof},
	},
	helper::initialize_wallet,
	parachains::relaychain::{
		self,
		api::{
			beefy_mmr_leaf::storage::types::beefy_authorities,
			runtime_apis::{beefy_api, beefy_mmr_api},
			runtime_types::{
				sp_consensus_beefy::{
					commitment::Commitment as spCommitment,
					ecdsa_crypto::{Public, Signature},
					payload::Payload,
					ForkVotingProof, FutureBlockVotingProof, VoteMessage,
				},
				sp_mmr_primitives::AncestryProof,
				sp_runtime::generic::{
					digest::{Digest, DigestItem},
					header::Header,
				},
				sp_session::MembershipProof,
			},
		},
	},
};
use subxt_signer::sr25519::dev;

use sp_consensus_beefy;
// use sp_consensus_beefy::ecdsa_crypto::Pair as PairT3;
use sp_core::{ecdsa::Pair, hex2array, Pair as PairT};
use sp_crypto_hashing::keccak_256;
use subxt::{
	client::OfflineClientT, ext::sp_core::hexdisplay::AsBytesRef, OnlineClient, PolkadotConfig,
};

// const GATEWAY_V2_ADDRESS: [u8; 20] = hex!("ee9170abfbf9421ad6dd07f6bdec9d89f2b581e0");

pub struct TestClients {
	pub relaychain_client: Box<OnlineClient<PolkadotConfig>>,
	pub ethereum_client: Box<Arc<Provider<Ws>>>,
	pub beefy_client: Box<BeefyClient<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>>,
}

async fn initialize_clients() -> Result<TestClients, Box<dyn std::error::Error>> {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await?
		.interval(Duration::from_millis(10u64));
	let ethereum_client = Arc::new(ethereum_provider);
	let ethereum_signed_client = Arc::new(initialize_wallet().await?);
	let relaychain_client = OnlineClient::from_url((*RELAY_CHAIN_WS_URL).to_string()).await?;
	let beefy_client_addr: Address = BEEFY_CLIENT_CONTRACT.into();
	let beefy_client = BeefyClient::new(beefy_client_addr, ethereum_signed_client.clone());
	Ok(TestClients {
		relaychain_client: Box::new(relaychain_client),
		ethereum_client: Box::new(ethereum_client),
		beefy_client: Box::new(beefy_client),
	})
}

fn validator_proof(
	signature_bytes: &[u8],
	signer_index: usize,
	validator_secp256k1_bytes: &Vec<[u8; 20]>,
	validator_proofs: &[[[u8; 32]; 2]; 4],
) -> ValidatorProof {
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
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
		index: U256::from_little_endian(&[signer_index.try_into().unwrap()]),
		account: H160::from_slice(&validator_secp256k1_bytes[signer_index]),
		proof: validator_proofs[signer_index].to_vec(),
	}
}

// TODO: reuse from polkadot-sdk
enum EquivocationType {
	FutureBlockEquivocation,
	ForkEquivocation,
}

#[tokio::test]
async fn malicious_payload() {
	// Setup clients
	// ---
	let test_clients = initialize_clients().await.expect("initialize clients");

	let (submit_initial, submit_final, report_equivocation) = (true, false, true);
	// let equivocation_type = EquivocationType::FutureBlockEquivocation;
	let equivocation_type = EquivocationType::ForkEquivocation;

	let current_validator_set = test_clients
		.beefy_client
		.current_validator_set()
		.call()
		.await
		.expect("beefy client initialized");

	let block_number = test_clients
		.beefy_client
		.latest_beefy_block()
		.call()
		.await
		.expect("beefy client initialized");

	println!("block_number: {:?}", block_number);

	let call = test_clients.beefy_client.latest_mmr_root();
	let current_mmr_root = call.call().await.expect("commit valid");
	println!("current mmr root: {:?}", current_mmr_root);
	let mut payload_data = Vec::new();
	payload_data.resize(32, 0);

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

	let payload = vec![PayloadItem { payload_id: [109, 104], data: Bytes::from(payload_data) }];
	let equivocation_block = match equivocation_type {
		EquivocationType::ForkEquivocation => (block_number as u32) + 1,
		EquivocationType::FutureBlockEquivocation => (block_number as u32) + 1000,
	};
	let commitment = Commitment {
		payload: payload.clone(),
		block_number: equivocation_block,
		validator_set_id: (current_validator_set.0 as u64),
	};

	let malicious_suris = vec!["//Westend04", "//Westend01", "//Westend02"];

	let malicious_authorities =
		malicious_suris.iter().map(|suri| Pair::from_string(suri, None).unwrap());
	println!(
		"malicious_authorities: {:?}",
		malicious_authorities.clone().map(|auth| auth.public()).collect::<Vec<_>>()
	);

	let sp_payload = sp_consensus_beefy::Payload::from_single_entry(
		payload[0].payload_id,
		payload[0].data.to_vec(),
	);
	let sp_commitment = sp_consensus_beefy::Commitment {
		payload: sp_payload,
		block_number: commitment.block_number,
		validator_set_id: commitment.validator_set_id,
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

	let bitfield: Vec<U256> = vec![U256::from_little_endian(&[0b0111])];

	let (validator_secp256k1_bytes, validator_set_root, validator_proofs) = {
		let validator_secp256k1_bytes = vec![
			hex2array!("fd4de54fb46fb25358323c12484dea951da5db48"),
			hex2array!("96fade2050ee5b75c01964e556b49a7c53de0bc5"),
			hex2array!("054426fc7aab50156c0dfcdcd607e7045cc58d9e"),
			hex2array!("a601c19ad010f21031f7317f08b4f0046db6ce2a"),
		];

		let keccak_validator_secp256k1_bytes: Vec<[u8; 32]> =
			validator_secp256k1_bytes.iter().map(|key| keccak_256(key)).collect();

		let keccak01 = keccak_256(
			&[keccak_validator_secp256k1_bytes[0], keccak_validator_secp256k1_bytes[1]].concat(),
		);
		let keccak23 = keccak_256(
			&[keccak_validator_secp256k1_bytes[2], keccak_validator_secp256k1_bytes[3]].concat(),
		);
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

	let signer_index = 0;
	let malicious_authority: Pair = Pair::from_string(malicious_suris[signer_index], None).unwrap();

	let proof = validator_proof(
		malicious_signatures[signer_index].0.as_slice(),
		signer_index,
		&validator_secp256k1_bytes,
		&validator_proofs,
	);

	if submit_initial {
		let call =
			test_clients
				.beefy_client
				.submit_initial(commitment.clone(), bitfield.clone(), proof);
		let result = call.send().await;

		println!("{:?}", result);
		if result.is_ok() {
			println!("success!");
		} else {
			println!("{:?}", result.as_ref().err().unwrap().as_revert().expect("is revert error"));
		}
		assert!(result.is_ok());
		tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

		let mut stream = test_clients.ethereum_client.subscribe_blocks().await.unwrap().take(3);

		while let Some(_block) = stream.next().await {}

		let call = test_clients.beefy_client.commit_prev_randao(*hashed_commitment);
		let result = call.send().await.expect("commit valid");
		println!("{:?}", result);
	}

	if submit_final {
		let mut stream = test_clients.ethereum_client.subscribe_blocks().await.unwrap().take(1);
		while let Some(_block) = stream.next().await {}

		let call = test_clients.beefy_client.create_final_bitfield(*hashed_commitment, bitfield);
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

		let mmr_leaf = Mmrleaf {
			version: 0,
			parent_number: 0,
			parent_hash: [0; 32],
			next_authority_set_id: (current_validator_set.0 as u64) + 1,
			next_authority_set_len: 4,
			next_authority_set_root: validator_set_root,
			parachain_heads_root: [0; 32],
		};

		let call = test_clients.beefy_client.submit_final(
			commitment,
			bitfield,
			chosen_malicious_proofs,
			mmr_leaf,
			vec![],
			U256::zero(),
		);
		let result = call.send().await;
		// println!("{:?}", result);
		assert!(result.is_ok());
	}


	if report_equivocation {
		// let beefy_storage_api = relaychain::api::beefy::storage::StorageApi;
		// let validator_set_id = beefy_storage_api.validator_set_id();
		// println!("validator_set_id: {:?}", validator_set_id);

		let key_ownership_proof = {
			let proof_query = beefy_api::BeefyApi::generate_key_ownership_proof(
				&beefy_api::BeefyApi,
				// validator_set_id,
				current_validator_set.0 as u64,
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
				let latest_block = test_clients
					.relaychain_client
					.blocks()
					.at_latest()
					.await
					.expect("can not connect to relaychain");
				let latest_header = latest_block.header();
				let encoded_header = latest_header.encode();
				let local_header: Header<_> =
					Header::decode_all(&mut encoded_header.as_bytes_ref())
						.expect("decode latest header");

				let genesis_hash = test_clients
					.relaychain_client
					.backend()
					.genesis_hash()
					.await
					.expect("get genesis hash");
				let ancestry_proof: AncestryProof<_> = {
					let ancestry_proof_query = beefy_api::BeefyApi::generate_ancestry_proof(
						&beefy_api::BeefyApi,
						equivocation_block - 10,
						None,
					);

					let api = test_clients.relaychain_client.runtime_api();


					let ancestry_proof_bytes = api
						.at_latest()
						.await
						.expect("can not connect to relaychain")
						.call(ancestry_proof_query)
						.await
						.expect("runtime query failed")
						.expect("validator set is not Some");

					AncestryProof::decode_all(&mut ancestry_proof_bytes.0.as_slice())
						.expect("decode ancestry proof")
				};

				let equivocation_proof = ForkVotingProof {
					ancestry_proof,
					header: local_header,
					vote: VoteMessage {
						commitment: spCommitment {
							block_number: sp_commitment.block_number,
							payload: Payload(vec![(
								payload[0].payload_id,
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

				let events = test_clients
					.relaychain_client
					.tx()
					.sign_and_submit_then_watch_default(&report, &dev::alice())
					.await
					.expect("submit report")
					.wait_for_finalized_success()
					.await
					.expect("finalized");

				println!("submitted fork equivocation report");

				events.find::<relaychain::api::offences::events::Offence>().for_each(|event| {
					println!("offence event: {event:?}");
				});
				events.find::<relaychain::api::staking::events::SlashReported>().for_each(
					|event| {
						println!("slash report event: {event:?}");
					},
				);
				events.find::<relaychain::api::staking::events::Slashed>().for_each(|event| {
					println!("slashed event: {event:?}");
				});
			},
			EquivocationType::FutureBlockEquivocation => {
				let equivocation_proof = FutureBlockVotingProof {
					vote: VoteMessage {
						commitment: spCommitment {
							block_number: sp_commitment.block_number,
							payload: Payload(vec![(
								payload[0].payload_id,
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

				let events = test_clients
					.relaychain_client
					.tx()
					.sign_and_submit_then_watch_default(&report, &dev::alice())
					.await
					.expect("submit report")
					.wait_for_finalized_success()
					.await
					.expect("finalized");

				events.find::<relaychain::api::offences::events::Offence>().for_each(|event| {
					println!("offence event: {event:?}");
				});
				events.find::<relaychain::api::staking::events::SlashReported>().for_each(
					|event| {
						println!("slash report event: {event:?}");
					},
				);
				events.find::<relaychain::api::staking::events::Slashed>().for_each(|event| {
					println!("slashed event: {event:?}");
				});
			},
		}
	}
}
