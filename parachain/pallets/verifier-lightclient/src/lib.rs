//! # Ethereum Light Client Verifier
//!
//! The verifier module implements verification of Ethereum transactions / events.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, ensure, traits::Get};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use artemis_ethereum::{HeaderId as EthereumHeaderId, H256, U256};
use artemis_ethereum::ethashproof::{DoubleNodeWithMerkleProof as EthashProofData, EthashProver};

pub use artemis_ethereum::Header as EthereumHeader;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Ethereum block header as it is stored in the runtime storage.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct StoredHeader<Submitter> {
	/// Submitter of this header. This will be None for the initial header
	/// or the account ID of the relay.
	pub submitter: Option<Submitter>,
	/// The block header itself.
	pub header: EthereumHeader,
	/// Total difficulty of the chain.
	pub total_difficulty: U256,
}

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	/// The number of descendents, in the highest difficulty chain, a block
	/// needs to have in order to be considered final.
	type DescendantsUntilFinalized: Get<u8>;
	// Determines whether Ethash PoW is verified for headers
	// NOTE: Should only be false for dev
	type VerifyPoW: Get<bool>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		/// Best known block.
		BestBlock: (EthereumHeaderId, U256);
		/// Best finalized block.
		FinalizedBlock: EthereumHeaderId;
		/// Map of imported headers by hash.
		Headers: map hasher(identity) H256 => Option<StoredHeader<T::AccountId>>;
		/// Map of imported header hashes by number.
		HeadersByNumber: map hasher(blake2_128_concat) u64 => Option<Vec<H256>>;
	}

	add_extra_genesis {
		config(initial_header): EthereumHeader;
		config(initial_difficulty): U256;

		build(|config| {
			let initial_header = &config.initial_header;
			let initial_hash = initial_header.compute_hash();
			let initial_id = EthereumHeaderId {
				number: initial_header.number,
				hash: initial_hash,
			};

			BestBlock::put((
				initial_id,
				config.initial_difficulty,
			));
			FinalizedBlock::put(initial_id);
			Headers::<T>::insert(
				initial_hash,
				StoredHeader {
					submitter: None,
					header: initial_header.clone(),
					total_difficulty: config.initial_difficulty,
				},
			);
			HeadersByNumber::insert(
				initial_header.number,
				vec![initial_hash],
			);
		})
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Header is same height or older than finalized block (we don't support forks).
		AncientHeader,
		/// Header's parent has not been imported.
		MissingParentHeader,
		/// Header has already been imported.
		DuplicateHeader,
		/// Header is on a stale fork, i.e. it's not a descendant of the latest finalized block
		HeaderOnStaleFork,
		/// One or more header fields are invalid.
		InvalidHeader,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		
		// TODO: Calculate weight
		#[weight = 0]
		pub fn import_header(origin, header: EthereumHeader, proof: Vec<EthashProofData>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::validate_header_to_import(&header, &proof)?;
			Self::import_validated_header(&sender, &header)
		}
	}
}

impl<T: Trait> Module<T> {
	// Validate an Ethereum header for import
	fn validate_header_to_import(header: &EthereumHeader, proof: &[EthashProofData]) -> DispatchResult {
		// TODO: move contextless check first
		let hash = header.compute_hash();
		ensure!(
			!Headers::<T>::contains_key(hash),
			Error::<T>::DuplicateHeader,
		);

		let parent = Headers::<T>::get(header.parent_hash)
			.ok_or(Error::<T>::MissingParentHeader)?
			.header;

		let finalized_header_id = FinalizedBlock::get();
		ensure!(
			header.number > finalized_header_id.number,
			Error::<T>::AncientHeader,
		);

		// TODO: limit N where N = header.number - finalized_header.number
		// to avoid iterating over long chains here
		for (ancestor_hash, ancestor) in ancestry::<T>(header.parent_hash) {
			if ancestor.number > finalized_header_id.number {
				continue;
			}
			// This assertion holds because the AncientHeader check above ensures that
			// iteration starts at or after the latest finalized block.
			assert_eq!(ancestor.number, finalized_header_id.number);
			ensure!(
				ancestor_hash == finalized_header_id.hash,
				Error::<T>::HeaderOnStaleFork,
			);
			break;
		}

		if !T::VerifyPoW::get() {
			return Ok(());
		}

		// Adapted from https://github.com/near/rainbow-bridge/blob/3fcdfbc6c0011f0e1507956a81c820616fb963b4/contracts/near/eth-client/src/lib.rs#L363
		// See YellowPaper formula (50) in section 4.3.4
		ensure!(
			header.gas_used <= header.gas_limit
			&& header.gas_limit < parent.gas_limit * 1025 / 1024
			&& header.gas_limit > parent.gas_limit * 1023 / 1024
			&& header.gas_limit >= 5000.into()
			&& header.timestamp > parent.timestamp
			&& header.number == parent.number + 1
			&& header.extra_data.len() <= 32,
			Error::<T>::InvalidHeader,
		);

		// Simplified difficulty check to conform adjusting difficulty bomb
		let header_mix_hash = header.mix_hash().ok_or(Error::<T>::InvalidHeader)?;
		let header_nonce = header.nonce().ok_or(Error::<T>::InvalidHeader)?;
		let (mix_hash, result) = EthashProver::new().hashimoto_merkle(
			header.compute_partial_hash(),
			header_nonce,
			header.number,
			proof,
		).map_err(|_| Error::<T>::InvalidHeader)?;
		ensure!(
			mix_hash == header_mix_hash
			&& U256::from(result.0) < ethash::cross_boundary(header.difficulty)
			&& header.difficulty < header.difficulty * 101 / 100
			&& header.difficulty > header.difficulty * 99 / 100,
			Error::<T>::InvalidHeader,
		);

		Ok(())
	}

	// Import a new, validated Ethereum header
	fn import_validated_header(sender: &T::AccountId, header: &EthereumHeader) -> DispatchResult {
		let hash = header.compute_hash();
		let stored_parent_header = Headers::<T>::get(header.parent_hash).unwrap();
		let total_difficulty = stored_parent_header.total_difficulty
			.checked_add(header.difficulty)
			.ok_or("Total difficulty overflow")?;
		let header_to_store = StoredHeader {
			submitter: Some(sender.clone()),
			header: header.clone(),
			total_difficulty,
		};

		Headers::<T>::insert(hash, header_to_store);

		if HeadersByNumber::contains_key(header.number) {
			HeadersByNumber::mutate(header.number, |option| {
				let hashes = option.as_mut().unwrap();
				hashes.push(hash);
			});
		} else {
			HeadersByNumber::insert(header.number, vec![hash]);
		}

		// Maybe track new highest difficulty chain
		let (_, highest_difficulty) = BestBlock::get();
		if total_difficulty > highest_difficulty {
			let best_block_id = EthereumHeaderId {
				number: header.number,
				hash,
			};
			BestBlock::put((best_block_id, total_difficulty));
			Self::finalize_headers(&best_block_id);
		}

		Ok(())
	}

	fn finalize_headers(best_block_id: &EthereumHeaderId) {
		let finalized_block_id = FinalizedBlock::get();
		let required_descendants = T::DescendantsUntilFinalized::get() as usize;
		for (i, (ancestor_hash, ancestor)) in ancestry::<T>(best_block_id.hash).enumerate() {
			if i < required_descendants {
				continue;
			}
			if ancestor.number > finalized_block_id.number {
				FinalizedBlock::put(EthereumHeaderId {
					hash: ancestor_hash,
					number: ancestor.number,
				});
			} else {
				assert!(ancestor_hash == finalized_block_id.hash);
			}
			break;
		}
	}
}

/// Return iterator over header ancestors, starting at given hash
fn ancestry<T: Trait>(mut hash: H256) -> impl Iterator<Item = (H256, EthereumHeader)> {
	sp_std::iter::from_fn(move || {
		let header = Headers::<T>::get(&hash)?.header;
		if header.number == 0 {
			return None;
		}

		let current_hash = hash;
		hash = header.parent_hash;
		Some((current_hash, header))
	})
}

// TODO implement artemis_core::Verifier
