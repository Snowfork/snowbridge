//! # Ethereum Light Client Verifier
//!
//! The verifier module implements verification of Ethereum transactions / events.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, ensure};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use artemis_ethereum::{HeaderId as EthereumHeaderId, H256, U256};

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
}

decl_storage! {
	trait Store for Module<T: Trait> as VerifierModule {
		/// Best known block.
		BestBlock: (EthereumHeaderId, U256);
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

			BestBlock::put((
				EthereumHeaderId {
					number: initial_header.number,
					hash: initial_hash,
				},
				config.initial_difficulty,
			));
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
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		
		#[weight = 0]
		pub fn import_header(origin, header: EthereumHeader) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::validate_header_to_import(&header)?;
			Self::import_validated_header(&sender, &header)
		}
	}
}

impl<T: Trait> Module<T> {
	// Validate an Ethereum header for import
	fn validate_header_to_import(header: &EthereumHeader) -> Result<(), &'static str> {
		ensure!(
			Headers::<T>::contains_key(header.parent_hash),
			"Parent header must be imported first",
		);

		let hash = header.compute_hash();
		ensure!(
			!Headers::<T>::contains_key(hash),
			"Header can only be imported once",
		);

		// TODO check PoW

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
			BestBlock::put((
				EthereumHeaderId {
					number: header.number,
					hash,
				},
				total_difficulty,
			));
		}

		Ok(())
	}
}

// TODO implement artemis_core::Verifier
