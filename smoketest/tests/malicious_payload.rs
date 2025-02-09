use std::{str::FromStr, sync::Arc, time::Duration};

use codec::Encode;
use ethers::{abi::Address, core::types::U256, prelude::*};
use snowbridge_smoketest::{
	constants::*,
	contracts::{
		beefy_client::{BeefyClient, Commitment, PayloadItem},
		shared_types::{Mmrleaf, ValidatorProof},
	},
	helper::initialize_wallet,
};
use sp_consensus_beefy;
use sp_core::{hex2array, ByteArray};
use sp_crypto_hashing::keccak_256;
use subxt::{
	client::OfflineClientT,
	ext::sp_core::{ecdsa::Pair, hexdisplay::AsBytesRef, Pair as PairT},
	OnlineClient, PolkadotConfig,
};

// TODO: replace
#[subxt::subxt(runtime_metadata_path = "./westend_relaychain_metadata.scale")]
pub mod westend {}

#[tokio::test]
async fn malicious_payload() {
	let ethereum_provider = Provider::<Ws>::connect((*ETHEREUM_API).to_string())
		.await
		.unwrap()
		.interval(Duration::from_millis(10u64));
	let ethereum_client = Arc::new(ethereum_provider);
	let ethereum_signed_client = Arc::new(initialize_wallet().await.expect("initialize wallet"));

	let relaychain_client: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*RELAY_CHAIN_WS_URL).to_string())
			.await
			.expect("can not connect to relaychain");

	let validator_set_id_query = westend::storage().beefy().validator_set_id();
	let validator_set_id = relaychain_client
		.storage()
		.at_latest()
		.await
		.expect("can not connect to relaychain")
		.fetch(&validator_set_id_query)
		.await
		.expect("runtime query failed")
		.expect("validator set is not Some");
	let block_number = relaychain_client
		.blocks()
		.at_latest()
		.await
		.expect("can not connect to relaychain")
		.number();

	let beefy_client_addr: Address = BEEFY_CLIENT_CONTRACT.into();
	let beefy_client = BeefyClient::new(beefy_client_addr, ethereum_signed_client.clone());

	let randao_delay = beefy_client
		.randao_commit_delay()
		.call()
		.await
		.expect("beefy client initialized");
	let payload = vec![PayloadItem { payload_id: [109, 104], data: Bytes::new() }];
	let commitment =
		Commitment { payload: payload.clone(), block_number: block_number + 10, validator_set_id };

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
	let encoded_commitment = sp_commitment.encode();
	println!("encoded commitment: {:?}", encoded_commitment);
	let hashed_commitment = &keccak_256(encoded_commitment.as_bytes_ref());
	println!("hashed commitment: {:?}", hashed_commitment);

	let malicious_signatures = malicious_authorities
		.map(|pair| pair.sign_prehashed(hashed_commitment))
		.collect::<Vec<_>>();

	println!(
		"malicious_signatures: {:?}",
		malicious_signatures.iter().map(|sig| sig.to_raw_vec()).collect::<Vec<_>>()
	);

	let bitfield: Vec<U256> = vec![U256::from_little_endian(&[0b0111])];

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

	let mut r = [0u8; 32];
	let mut s = [0u8; 32];

	for signer_index in 0..=0 {
		let init_signature_bytes = malicious_signatures[signer_index].as_slice();
		r.copy_from_slice(&init_signature_bytes[0..32]);
		s.copy_from_slice(&init_signature_bytes[32..64]);

		// For legacy format, convert 0/1 to 27/28
		let v_raw = init_signature_bytes[64];
		let v = match v_raw {
			0 => 27,
			1 => 28,
			_ => panic!("v can only be 0 or 1"),
		};
		let proof = ValidatorProof {
			v,
			r,
			s,
			index: U256::from_little_endian(&[signer_index.try_into().unwrap()]),
			account: H160::from_slice(&validator_secp256k1_bytes[signer_index]),
			proof: validator_proofs[signer_index].to_vec(),
		};

		let call = beefy_client.submit_initial(commitment.clone(), bitfield.clone(), proof);
		let result = call.send().await;

		assert!(result.is_ok());
		tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
	}

	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(3);

	while let Some(_block) = stream.next().await {}

	let call = beefy_client.commit_prev_randao(*hashed_commitment);
	let result = call.send().await.expect("commit valid");
	println!("{:?}", result);

	let mut stream = ethereum_client.subscribe_blocks().await.unwrap().take(1);
	while let Some(_block) = stream.next().await {}

	let call = beefy_client.create_final_bitfield(*hashed_commitment, bitfield);
	let bitfield = call.call().await.expect("commit valid");

	assert_eq!(bitfield.len(), 1);
	let chosen_malicious_proofs = malicious_signatures
		.iter()
		.enumerate()
		.filter(|(i, _)| bitfield[0].bit(*i))
		.map(|(i, sig)| {
			//TODO: deduplicate with init sig
			let sig_bytes = sig.as_slice();

			r.copy_from_slice(&sig_bytes[0..32]);
			s.copy_from_slice(&sig_bytes[32..64]);

			let v_raw = sig_bytes[64];
			let v = match v_raw {
				0 => 27,
				1 => 28,
				_ => panic!("v can only be 0 or 1"),
			};

			ValidatorProof {
				v,
				r,
				s,
				index: U256::from_little_endian(&[i.try_into().unwrap()]),
				account: H160::from_slice(&validator_secp256k1_bytes[i]),
				proof: validator_proofs[i].to_vec(),
			}
		})
		.collect::<Vec<_>>();

	let mmr_leaf = Mmrleaf {
		version: 0,
		parent_number: 0,
		parent_hash: [0; 32],
		next_authority_set_id: validator_set_id + 1,
		next_authority_set_len: 4,
		next_authority_set_root: validator_set_root,
		parachain_heads_root: [0; 32],
	};

	let call = beefy_client.submit_final(
		commitment,
		bitfield,
		chosen_malicious_proofs,
		mmr_leaf,
		vec![],
		U256::zero(),
	);
	let result = call.send().await;
	println!("{:?}", result);

	// verify: error is `InvalidMMRRootLength` (selector 0x7df9c486)
	assert_eq!(
		result.as_ref().err().unwrap().as_revert().expect("is revert error"),
		&Bytes::from_str("0x7df9c486").unwrap()
	);
}
