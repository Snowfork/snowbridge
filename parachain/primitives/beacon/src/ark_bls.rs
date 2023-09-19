// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>

// core
use core::{borrow::Borrow, ops::Neg};
// crates.io
use ark_bls12_381::{
	g2::Config as G2Config, Bls12_381, G1Affine, G1Projective, G2Affine, G2Projective,
};
use ark_ec::{
	hashing::{
		curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve,
		HashToCurveError,
	},
	models::short_weierstrass::Projective,
	pairing::Pairing,
	AffineRepr, CurveGroup,
};
use ark_ff::{field_hashers::DefaultFieldHasher, Zero};
use ark_serialize::*;
use sha2::Sha256;
// substrate
use crate::types::BlsError;
use sp_std::prelude::Vec;

/// Domain Separation Tag for signatures on G2
pub const DST_G2: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

#[derive(Clone, Debug)]
pub struct Signature(G2Projective);
impl From<G2Projective> for Signature {
	fn from(sig: G2Projective) -> Signature {
		Signature(sig)
	}
}
impl AsRef<G2Projective> for Signature {
	fn as_ref(&self) -> &G2Projective {
		&self.0
	}
}
impl Signature {
	pub fn from_bytes(bytes: &[u8]) -> Result<Signature, SerializationError> {
		let p = G2Affine::deserialize_compressed(bytes)?;
		Ok(Self(p.into()))
	}

	#[allow(dead_code)]
	pub fn aggregate<S: Borrow<Signature>>(signatures: impl IntoIterator<Item = S>) -> Signature {
		signatures.into_iter().map(|s| s.borrow().0).sum::<G2Projective>().into()
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, CanonicalSerialize, CanonicalDeserialize)]
pub struct PublicKey(pub G1Projective);
impl From<G1Projective> for PublicKey {
	fn from(pk: G1Projective) -> PublicKey {
		PublicKey(pk)
	}
}
impl PublicKey {
	pub fn from_bytes(bytes: &[u8]) -> Result<PublicKey, SerializationError> {
		let p = G1Affine::deserialize_compressed_unchecked(bytes)?;
		Ok(Self(p.into()))
	}

	pub fn aggregate<P: Borrow<PublicKey>>(public_keys: impl IntoIterator<Item = P>) -> PublicKey {
		public_keys.into_iter().map(|s| s.borrow().0).sum::<G1Projective>().into()
	}

	pub fn verify(&self, signature: &Signature, message: &G2Projective) -> bool {
		Bls12_381::multi_pairing(
			[G1Affine::generator().neg(), self.0.into_affine()],
			[signature.as_ref().into_affine(), message.into_affine()],
		)
		.is_zero()
	}
}

pub fn hash_to_curve_g2(message: &[u8]) -> Result<G2Projective, HashToCurveError> {
	let wb_to_curve_hasher = MapToCurveBasedHasher::<
		Projective<G2Config>,
		DefaultFieldHasher<Sha256, 128>,
		WBMap<G2Config>,
	>::new(DST_G2)?;
	Ok(wb_to_curve_hasher.hash(message)?.into())
}

pub fn fast_aggregate_verify(
	pub_keys: Vec<Vec<u8>>,
	message: Vec<u8>,
	signature: Vec<u8>,
) -> Result<bool, BlsError> {
	let sig = Signature::from_bytes(&signature).map_err(|_| BlsError::InvalidSignature)?;
	let pub_keys: Result<Vec<PublicKey>, _> =
		pub_keys.into_iter().map(|pk| PublicKey::from_bytes(&pk)).collect();
	let Ok(pks) = pub_keys else { return Err(BlsError::InvalidPublicKey) };

	let agg_pk = PublicKey::aggregate(pks);
	let msg = hash_to_curve_g2(&message).map_err(|_| BlsError::HashToCurveFailed)?;
	Ok(agg_pk.verify(&sig, &msg))
}

#[cfg(test)]
mod tests {
	// crates.io
	use super::*;
	use ark_bls12_381::Fr;
	use ark_ec::Group;
	use ark_ff::UniformRand;
	use ark_std::test_rng;
	use rand::Rng;

	#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
	pub struct SecretKey(Fr);

	impl From<Fr> for SecretKey {
		fn from(sk: Fr) -> SecretKey {
			SecretKey(sk)
		}
	}

	impl From<&SecretKey> for PublicKey {
		fn from(sk: &SecretKey) -> PublicKey {
			(G1Projective::generator() * sk.as_ref()).into()
		}
	}

	impl AsRef<Fr> for SecretKey {
		fn as_ref(&self) -> &Fr {
			&self.0
		}
	}

	impl SecretKey {
		pub fn new<R: Rng>(rng: &mut R) -> SecretKey {
			SecretKey(Fr::rand(rng))
		}

		pub fn sign(&self, message: &G2Projective) -> Signature {
			(*message * self.as_ref()).into()
		}
	}

	#[test]
	fn test_verify() {
		let rng = &mut test_rng();
		let message = G2Projective::rand(rng);

		let sks = (0..10).map(|_| SecretKey::new(rng)).collect::<Vec<_>>();
		let pks = sks.iter().map(PublicKey::from).collect::<Vec<_>>();
		let sigs = sks.iter().map(|sk| sk.sign(&message)).collect::<Vec<_>>();
		pks.iter()
			.zip(sigs.iter())
			.for_each(|(pk, sig)| assert!(pk.verify(sig, &message)));

		let apk = PublicKey::aggregate(pks);
		let asig = Signature::aggregate(sigs);
		assert!(apk.verify(&asig, &message));
	}

	#[test]
	fn test_hash_to_curve() {
		let message = vec![
			58, 137, 108, 164, 181, 219, 16, 43, 157, 253, 71, 82, 139, 6, 34, 10, 145, 189, 18,
			70, 29, 204, 134, 121, 60, 226, 213, 145, 244, 30, 164, 248,
		];
		let e = vec![
			178, 18, 44, 225, 215, 170, 68, 228, 52, 151, 40, 113, 171, 202, 76, 203, 156, 112,
			105, 249, 147, 210, 132, 79, 69, 117, 109, 151, 35, 71, 117, 21, 119, 179, 181, 81, 92,
			22, 22, 88, 190, 243, 147, 248, 3, 210, 87, 98, 0, 84, 201, 248, 182, 249, 99, 59, 86,
			60, 71, 244, 250, 189, 134, 232, 18, 82, 72, 76, 83, 155, 46, 113, 128, 107, 49, 67,
			174, 100, 244, 181, 33, 174, 14, 151, 112, 62, 141, 100, 173, 191, 103, 178, 205, 17,
			237, 147,
		];
		let p: G2Affine = hash_to_curve_g2(&message).unwrap().into();
		let mut c = Vec::new();
		p.serialize_compressed(&mut c).unwrap();
		assert_eq!(e, c);
	}
}
