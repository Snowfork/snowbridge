use std::sync::Arc;

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
use sp_crypto_hashing::keccak_256;
use subxt::ext::sp_core::{ecdsa::Pair, hexdisplay::AsBytesRef, Pair as PairT};

#[tokio::test]
async fn malicious_payload() {
	let ethereum_client = Arc::new(initialize_wallet().await.expect("initialize wallet"));

	let beefy_client_addr: Address = BEEFY_CLIENT_CONTRACT.into();
	let beefy_client = BeefyClient::new(beefy_client_addr, ethereum_client.clone());

	let payload = vec![PayloadItem { payload_id: [0, 0], data: Bytes::new() }];
	let commitment =
		Commitment { payload: payload.clone(), block_number: 50000, validator_set_id: 0 };

	let malicious_suris = vec!["//Westend01", "//Westend02", "//Westend03"];
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
	let init_signature = malicious_signatures[0].clone();
	println!("init_signature: {:?}", init_signature);

	let bitfield: Vec<U256> = vec![U256::one(), U256::one(), U256::one(), U256::zero()];

	let proof = ValidatorProof {
		v: 0,
		r: [0; 32],
		s: [0; 32],
		index: U256::zero(),
		account: H160::zero(),
		proof: vec![[0; 32]],
	};

	let call = beefy_client.submit_initial(commitment, bitfield, proof);
	let result = call.send().await;
	// verify: error is `InvalidCommitment` (selector 0xc06789fa)
	assert_eq!(
		result.err().unwrap().as_revert().expect("is revert error"),
		&Bytes::from_hex("c06789fa").unwrap()
	);
}
