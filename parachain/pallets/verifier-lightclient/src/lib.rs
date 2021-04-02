//! # Ethereum Light Client Verifier
//!
//! The verifier module implements verification of Ethereum transactions / events.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{
	debug, decl_module, decl_storage, decl_event, decl_error, ensure,
	dispatch::{DispatchError, DispatchResult},
	traits::Get, weights::Weight,
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use artemis_core::{Message, Verifier, Proof};
use artemis_ethereum::{
	HeaderId as EthereumHeaderId, Log, Receipt, H256, U256,
	difficulty::calc_difficulty,
	ethashproof::{DoubleNodeWithMerkleProof as EthashProofData, EthashProver},
};
pub use artemis_ethereum::{
	Header as EthereumHeader, difficulty::DifficultyConfig as EthereumDifficultyConfig,
};

mod benchmarking;

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

/// Weight functions needed for this pallet.
pub trait WeightInfo {
	fn import_header() -> Weight;
	fn import_header_not_new_finalized_with_max_prune() -> Weight;
	fn import_header_new_finalized_with_single_prune() -> Weight;
	fn import_header_not_new_finalized_with_single_prune() -> Weight;
}

impl WeightInfo for () {
	fn import_header() -> Weight { 0 }
	fn import_header_not_new_finalized_with_max_prune() -> Weight { 0 }
	fn import_header_new_finalized_with_single_prune() -> Weight { 0 }
	fn import_header_not_new_finalized_with_single_prune() -> Weight { 0 }
}

pub trait Config: system::Config {
	type Event: From<Event> + Into<<Self as system::Config>::Event>;
	/// The number of descendants, in the highest difficulty chain, a block
	/// needs to have in order to be considered final.
	type DescendantsUntilFinalized: Get<u8>;
	/// Ethereum network parameters for header difficulty
	type DifficultyConfig: Get<EthereumDifficultyConfig>;
	/// Determines whether Ethash PoW is verified for headers
	/// NOTE: Should only be false for dev
	type VerifyPoW: Get<bool>;
	/// Weight information for extrinsics in this pallet
	type WeightInfo: WeightInfo;
}

decl_storage! {
	trait Store for Module<T: Config> as VerifierLightclient {
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

			Module::<T>::initialize_storage(
				vec![initial_header.clone()],
				config.initial_difficulty,
				0, // descendants_until_final = 0 forces the initial header to be finalized
			).unwrap();

			BlocksToPrune::put(PruningRange {
				oldest_unpruned_block: initial_header.number,
				oldest_block_to_keep: initial_header.number,
			});
		})
	}
}

decl_event!(
	pub enum Event {

	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
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
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = T::WeightInfo::import_header()]
		pub fn import_header(origin, header: EthereumHeader, proof: Vec<EthashProofData>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			debug::RuntimeLogger::init();

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

impl<T: Config> Module<T> {
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

		// This iterates over DescendantsUntilFinalized headers in both the worst and
		// average case. Since we know that the parent header was imported successfully,
		// we know that the newest finalized header is at most, and on average,
		// DescendantsUntilFinalized headers before the parent.
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

		let difficulty_config = T::DifficultyConfig::get();
		let header_difficulty = calc_difficulty(&difficulty_config, header.timestamp, &parent)
			.map_err(|_| Error::<T>::InvalidHeader)?;
		ensure!(
			header.difficulty == header_difficulty,
			Error::<T>::InvalidHeader,
		);

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
			&& U256::from(result.0) < ethash::cross_boundary(header.difficulty),
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
		if total_difficulty > highest_difficulty || (!T::VerifyPoW::get() && total_difficulty == U256::zero()) {
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

	// Verifies that the receipt encoded in proof.data is included
	// in the block given by proof.block_hash. Inclusion is only
	// recognized if the block has been finalized.
	fn verify_receipt_inclusion(proof: &Proof) -> Result<Receipt, DispatchError> {
		let header = Headers::<T>::get(proof.block_hash)
			.ok_or(Error::<T>::MissingHeader)?
			.header;

		let receipt = header.check_receipt_proof(&proof.data.1)
			.ok_or(Error::<T>::InvalidProof)?;

		let finalized_block = FinalizedBlock::get();
		Self::check_header_finality(
			&EthereumHeaderId { hash: proof.block_hash, number: header.number },
			&finalized_block,
		)?;

		Ok(receipt)
	}

	// Checks that the header is in the finalized chain
	fn check_header_finality(
		header: &EthereumHeaderId,
		finalized_block: &EthereumHeaderId,
	) -> DispatchResult {
		if header.number == finalized_block.number {
			return if header.hash == finalized_block.hash {
				Ok(())
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
			header.hash == finalized_hash_at_number,
			Error::<T>::HeaderOnStaleFork,
		);

		Ok(())
	}
}

/// Return iterator over header ancestors, starting at given hash
fn ancestry<T: Config>(mut hash: H256) -> impl Iterator<Item = (H256, EthereumHeader)> {
	sp_std::iter::from_fn(move || {
		let header = Headers::<T>::get(&hash)?.header;
		let current_hash = hash;
		hash = header.parent_hash;
		Some((current_hash, header))
	})
}

impl<T: Config> Verifier for Module<T> {

	fn verify(message: &Message) -> Result<Log, DispatchError> {
		let receipt = Self::verify_receipt_inclusion(&message.proof)?;

		let log: Log = rlp::decode(&message.data)
			.map_err(|_| Error::<T>::InvalidProof)?;

		if !receipt.contains_log(&log) {
			return Err(Error::<T>::InvalidProof.into());
		}

		Ok(log)
	}

	fn initialize_storage(
		headers: Vec<EthereumHeader>,
		initial_difficulty: U256,
		descendants_until_final: u8,
	) -> Result<(), &'static str> {
		let insert_header_fn = |header: &EthereumHeader, total_difficulty: U256| {
			let hash = header.compute_hash();
			Headers::<T>::insert(
				hash,
				StoredHeader {
					submitter: None,
					header: header.clone(),
					total_difficulty: total_difficulty,
				},
			);
			HeadersByNumber::append(header.number, hash);

			EthereumHeaderId {
				number: header.number,
				hash: hash,
			}
		};

		let oldest_header = headers.get(0).ok_or("Need at least one header")?;
		let mut best_block_difficulty = initial_difficulty;
		let mut best_block_id = insert_header_fn(&oldest_header, best_block_difficulty);

		for (i, header) in headers.iter().enumerate().skip(1) {
			let prev_block_num = headers[i - 1].number;
			ensure!(
				header.number == prev_block_num || header.number == prev_block_num + 1,
				"Headers must be in order",
			);

			let total_difficulty = {
				let parent = Headers::<T>::get(header.parent_hash).ok_or("Missing parent header")?;
				parent.total_difficulty + header.difficulty
			};

			let block_id = insert_header_fn(&header, total_difficulty);

			if total_difficulty > best_block_difficulty {
				best_block_difficulty = total_difficulty;
				best_block_id = block_id;
			}
		}

		BestBlock::put((best_block_id, best_block_difficulty));

		let maybe_finalized_ancestor = ancestry::<T>(best_block_id.hash)
			.enumerate()
			.find_map(|(i, pair)| if i < descendants_until_final as usize { None } else { Some(pair) });
		if let Some((hash, header)) = maybe_finalized_ancestor {
			FinalizedBlock::put(EthereumHeaderId {
				hash: hash,
				number: header.number,
			});
		}

		Ok(())
	}
}
