//! # Ethereum Light Client Verifier
//!
//! The verifier module implements verification of Ethereum transactions / events.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{debug, decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, dispatch::DispatchError, ensure, traits::Get};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use artemis_core::{AppId, Message, Verifier, VerificationInput, VerificationOutput};
use artemis_ethereum::{HeaderId as EthereumHeaderId, Receipt, H256, U256};
use artemis_ethereum::ethashproof::{DoubleNodeWithMerkleProof as EthashProofData, EthashProver};

pub use artemis_ethereum::Header as EthereumHeader;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Max number of finalized headers to keep.
const FINALIZED_HEADERS_TO_KEEP: u64 = 5_000;
/// Max number of headers we're pruning in single import call.
const HEADERS_TO_PRUNE_IN_SINGLE_IMPORT: u64 = 8;

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

/// Blocks range that we want to prune.
#[derive(Clone, Encode, Decode, Default, PartialEq, RuntimeDebug)]
struct PruningRange {
	/// Number of the oldest unpruned block(s). This might be the block that we do not
	/// want to prune now (then it is equal to `oldest_block_to_keep`).
	pub oldest_unpruned_block: u64,
	/// Number of oldest block(s) that we want to keep. We want to prune blocks in range
	/// [`oldest_unpruned_block`; `oldest_block_to_keep`).
	pub oldest_block_to_keep: u64,
}

pub trait Trait: system::Trait {
	type Event: From<Event> + Into<<Self as system::Trait>::Event>;
	/// The number of descendants, in the highest difficulty chain, a block
	/// needs to have in order to be considered final.
	type DescendantsUntilFinalized: Get<u8>;
	// Determines whether Ethash PoW is verified for headers
	// NOTE: Should only be false for dev
	type VerifyPoW: Get<bool>;
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierLightclient {
		/// Best known block.
		BestBlock: (EthereumHeaderId, U256);
		/// Range of blocks that we want to prune.
		BlocksToPrune: PruningRange;
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
			BlocksToPrune::put(PruningRange {
				oldest_unpruned_block: initial_id.number,
				oldest_block_to_keep: initial_id.number,
			});
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
		/// Header referenced in inclusion proof doesn't exist, e.g. because it's
		/// pruned or older than genesis.
		MissingHeader,
		/// Header's parent has not been imported.
		MissingParentHeader,
		/// Header has already been imported.
		DuplicateHeader,
		/// Header referenced in inclusion proof is not final yet.
		HeaderNotFinalized,
		/// Header is on a stale fork, i.e. it's not a descendant of the latest finalized block
		HeaderOnStaleFork,
		/// One or more header fields are invalid.
		InvalidHeader,
		/// Proof could not be applied / verified.
		InvalidProof,
		/// This should never be returned - indicates a bug
		Unknown,
		/// The specified verification scheme is not implemented by this module.
		UnsupportedVerificationScheme,
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

			debug::trace!(
				target: "import_header",
				"Received header {}. Starting validation",
				header.number,
			);

			if let Err(err) = Self::validate_header_to_import(&header, &proof) {
				debug::trace!(
					target: "import_header",
					"Validation for header {} returned error. Skipping import",
					header.number,
				);
				return Err(err);
			}
		
			debug::trace!(
				target: "import_header",
				"Validation succeeded. Starting import of header {}",
				header.number,
			);

			if let Err(err) = Self::import_validated_header(&sender, &header) {
				debug::trace!(
					target: "import_header",
					"Import of header {} failed",
					header.number,
				);
				return Err(err);
			}

			debug::trace!(
				target: "import_header",
				"Import of header {} succeeded!",
				header.number,
			);

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	// Validate an Ethereum header for import
	fn validate_header_to_import(header: &EthereumHeader, proof: &[EthashProofData]) -> DispatchResult {
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
		let ancestor_at_finalized_number = ancestry::<T>(header.parent_hash)
			.find(|(_, ancestor)| ancestor.number == finalized_header_id.number);
		// We must find a matching ancestor above since AncientHeader check ensures
		// that iteration starts at or after the latest finalized block.
		ensure!(
			ancestor_at_finalized_number.is_some(),
			Error::<T>::Unknown,
		);
		ensure!(
			ancestor_at_finalized_number.unwrap().0 == finalized_header_id.hash,
			Error::<T>::HeaderOnStaleFork,
		);
	
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
		let stored_parent_header = Headers::<T>::get(header.parent_hash)
			.ok_or(Error::<T>::MissingParentHeader)?;
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
			let mut mutate_failed = false;
			HeadersByNumber::mutate(header.number, |option| {
				match option.as_mut() {
					Some(hashes) => hashes.push(hash),
					None => mutate_failed = true,
				}
			});
			ensure!(!mutate_failed, Error::<T>::Unknown);
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

			// Finalize blocks if possible
			let finalized_block_id = FinalizedBlock::get();
			let new_finalized_block_id = Self::get_best_finalized_header(
				&best_block_id,
				&finalized_block_id,
			)?;
			if new_finalized_block_id != finalized_block_id {
				FinalizedBlock::put(new_finalized_block_id);
			}
	
			// Clean up old headers
			let pruning_range = BlocksToPrune::get();
			let new_pruning_range = Self::prune_header_range(
				&pruning_range,
				HEADERS_TO_PRUNE_IN_SINGLE_IMPORT,
				new_finalized_block_id.number.saturating_sub(FINALIZED_HEADERS_TO_KEEP),
			);
			if new_pruning_range != pruning_range {
				BlocksToPrune::put(new_pruning_range);
			}
		}

		Ok(())
	}

	// Return the latest block that can be finalized based on the given
	// highest difficulty chain and previously finalized block.
	fn get_best_finalized_header(
		best_block_id: &EthereumHeaderId,
		finalized_block_id: &EthereumHeaderId,
	) -> Result<EthereumHeaderId, DispatchError> {
		let required_descendants = T::DescendantsUntilFinalized::get() as usize;
		let maybe_newly_finalized_ancestor = ancestry::<T>(best_block_id.hash)
			.enumerate()
			.find_map(|(i, pair)| if i < required_descendants { None } else { Some(pair) });
		
		match maybe_newly_finalized_ancestor {
			Some((hash, header)) => {
				// The header is newly finalized if it is younger than the current
				// finalized block
				if header.number > finalized_block_id.number {
					return Ok(EthereumHeaderId {
						hash: hash,
						number: header.number,
					});
				}
				if hash != finalized_block_id.hash {
					return Err(Error::<T>::Unknown.into());
				}
				Ok(finalized_block_id.clone())
			}
			None => Ok(finalized_block_id.clone())
		}
	}

	// Remove old headers, from oldest to newest, in the provided range
	// (adjusted to `prune_end` if newer). Only up to `max_headers_to_prune`
	// will be removed.
	fn prune_header_range(
		pruning_range: &PruningRange,
		max_headers_to_prune: u64,
		prune_end: u64,
	) -> PruningRange {
		let mut new_pruning_range = pruning_range.clone();

		// We can only increase this since pruning cannot be reverted...
		if prune_end > new_pruning_range.oldest_block_to_keep {
			new_pruning_range.oldest_block_to_keep = prune_end;
		}

		let start = new_pruning_range.oldest_unpruned_block;
		let end = new_pruning_range.oldest_block_to_keep;
		let mut blocks_pruned = 0;
		for number in start..end {
			if blocks_pruned == max_headers_to_prune {
				break;
			}

			if let Some(hashes_at_number) = HeadersByNumber::take(number) {
				let mut remaining = hashes_at_number.len();
				for hash in hashes_at_number.iter() {
					Headers::<T>::remove(hash);
					blocks_pruned += 1;
					remaining -= 1;
					if blocks_pruned == max_headers_to_prune {
						break;
					}
				}

				if remaining > 0 {
					let remainder = &hashes_at_number[hashes_at_number.len() - remaining..];
					HeadersByNumber::insert(number, remainder);
				} else {
					new_pruning_range.oldest_unpruned_block = number + 1;
				}
			} else {
				new_pruning_range.oldest_unpruned_block = number + 1;
			}
		}

		new_pruning_range
	}

	fn verify_receipt_inclusion(block_hash: &H256, proof: &[Vec<u8>]) -> Result<Receipt, DispatchError> {
		let header = Headers::<T>::get(block_hash)
			.ok_or(Error::<T>::MissingHeader)?
			.header;
	
		let receipt = header.check_receipt_proof(&proof)
			.ok_or(Error::<T>::InvalidProof)?;
		
		// Check that the header is in the finalized chain
		let finalized_block = FinalizedBlock::get();
	
		if header.number == finalized_block.number {
			return if *block_hash == finalized_block.hash {
				Ok(receipt)
			} else {
				Err(Error::<T>::HeaderOnStaleFork.into())
			}
		}
	
		ensure!(
			header.number < finalized_block.number,
			Error::<T>::HeaderNotFinalized,
		);
		
		let (finalized_hash_at_number, _) = ancestry::<T>(finalized_block.hash)
			.find(|(_, ancestor)| ancestor.number == header.number)
			.ok_or(Error::<T>::HeaderOnStaleFork)?;
		ensure!(
			*block_hash == finalized_hash_at_number,
			Error::<T>::HeaderOnStaleFork,
		);
	
		Ok(receipt)
	}
}

/// Return iterator over header ancestors, starting at given hash
fn ancestry<T: Trait>(mut hash: H256) -> impl Iterator<Item = (H256, EthereumHeader)> {
	sp_std::iter::from_fn(move || {
		let header = Headers::<T>::get(&hash)?.header;
		let current_hash = hash;
		hash = header.parent_hash;
		Some((current_hash, header))
	})
}

impl<T: Trait> Verifier<T::AccountId> for Module<T> {

	fn verify(_: T::AccountId, _: AppId, message: &Message) -> Result<VerificationOutput, DispatchError> {
		match message.verification {
			VerificationInput::ReceiptProof { ref block_hash, ref tx_index, ref proof } => {
				let receipt = Self::verify_receipt_inclusion(block_hash, &proof.1)?;
				Ok(VerificationOutput::Receipt(receipt))
			},
			_ => Err(Error::<T>::UnsupportedVerificationScheme.into())
		}
	}
}
