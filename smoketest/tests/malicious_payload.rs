use std::{str::FromStr, sync::Arc};

use codec::Encode;
use ethers::{abi::Address, core::types::U256, prelude::*};
use hex::FromHex;
use snowbridge_smoketest::{
	constants::*,
	contracts::{
		beefy_client::{BeefyClient, Commitment, PayloadItem},
		shared_types::ValidatorProof,
	},
	helper::initialize_wallet,
};
use sp_consensus_beefy;
use sp_core::ByteArray;
use sp_crypto_hashing::keccak_256;
use subxt::{
	client::OfflineClientT,
	ext::sp_core::{ecdsa::Pair, hexdisplay::AsBytesRef, Pair as PairT},
	OnlineClient, PolkadotConfig,
};

// TODO: replace
#[subxt::subxt(runtime_metadata_path = "./polkadot_relaychain_metadata.scale")]
pub mod polkadot {}

#[tokio::test]
async fn malicious_payload() {
	let ethereum_client = Arc::new(initialize_wallet().await.expect("initialize wallet"));

	let relaychain_client: OnlineClient<PolkadotConfig> =
		OnlineClient::from_url((*RELAY_CHAIN_WS_URL).to_string())
			.await
			.expect("can not connect to relaychain");

	let validator_set_id_query = polkadot::storage().beefy().validator_set_id();
	let validator_set_id = relaychain_client
		.storage()
		.at_latest()
		.await
		.expect("can not connect to relaychain")
		.fetch(&validator_set_id_query)
		.await
		.expect("runtime query failed")
		.expect("validator set is not Some");
	let beefy_client_addr: Address = BEEFY_CLIENT_CONTRACT.into();
	let beefy_client = BeefyClient::new(beefy_client_addr, ethereum_client.clone());

	let payload = vec![PayloadItem { payload_id: [0, 0], data: Bytes::new() }];
	let commitment = Commitment { payload: payload.clone(), block_number: 50000, validator_set_id };

	let malicious_suris = vec!["//Westend01", "//Westend02", "//Westend03", "//Westend04"];
	let malicious_authorities =
		malicious_suris.iter().map(|suri| Pair::from_string(suri, None).unwrap());
	println!("malicious_authorities: {:?}", malicious_authorities);

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

	println!("malicious_signatures: {:?}", malicious_signatures);
	let init_signature = malicious_signatures[3].clone();
	println!("init_signature: {:?}", init_signature);

	let init_signature_bytes = init_signature.as_slice();
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];
	r.copy_from_slice(&init_signature_bytes[0..32]);
	s.copy_from_slice(&init_signature_bytes[32..64]);
	let v = init_signature_bytes[64];
	let v = match v {
		0 => 27,
		1 => 28,
		_ => 0,
	};

	let bitfield: Vec<U256> = vec![U256::one(), U256::one(), U256::one(), U256::zero()];

	let proof = ValidatorProof {
		v,
		r,
		s,
		index: U256::zero(),
		// hardcoded 0th validator account
		account: H160::from_str("fd4de54fb46fb25358323c12484dea951da5db48").expect("valid address"),
		// hardcoded 0th validator merkle proof proof in static authority set
		proof: vec![
			[
				5, 120, 92, 249, 72, 54, 128, 155, 50, 161, 184, 237, 9, 152, 81, 248, 77, 238, 54,
				114, 159, 19, 59, 166, 156, 6, 153, 35, 145, 193, 253, 220,
			],
			[
				169, 40, 149, 147, 19, 125, 50, 4, 149, 113, 52, 71, 100, 184, 239, 180, 111, 163,
				176, 168, 111, 129, 204, 149, 156, 13, 30, 206, 185, 9, 38, 134,
			],
		],
	};

	let call = beefy_client.submit_initial(commitment, bitfield, proof);
	let result = call.send().await;
	assert!(result.is_ok());
}
