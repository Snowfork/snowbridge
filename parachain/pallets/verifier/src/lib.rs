//! # Verifier
//!
//! The verifier module provides functionality for message verification.
//!
//! ## Overview
//!
//! This verifier performs the following verification routines on a message:
//! - Ensuring that the message sender is trusted
//! - Ensuring that messages are not replayed
//!
//! This verifier is intended to be swapped out for an Ethereum light-client solution at some point.
//!
//! ## Interface
//!
//! The verifier implements the [`Verifier`] trait and conforms to its interface.
//!
#![allow(unused_variables)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::{self as system, ensure_signed};
use frame_support::{decl_module, decl_storage, decl_event, decl_error,
	dispatch::DispatchResult, ensure};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use codec::{Encode, Decode};

use artemis_core::{AppId, Message, Verifier, VerificationInput};
use artemis_ethereum::{HeaderId as EthereumHeaderId, H256, U256};

pub use artemis_ethereum::Header as EthereumHeader;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Ethereum block header as it is stored in the runtime storage.
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct StoredEthereumHeader<Submitter> {
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
		/// The trusted [`AccountId`] of the external relayer service.
		RelayKey get(fn key) config(): T::AccountId;

		/// Hashes of previously seen messages. Used to implement replay protection.
		pub VerifiedPayloads: map hasher(blake2_128_concat) T::Hash => ();
 
 		/// START: Fields that track the Ethereum chain

 		/// Best known block.
		EthBestBlock: (EthereumHeaderId, U256);
		/// Map of imported headers by hash.
		EthHeaders: map hasher(identity) H256 => Option<StoredEthereumHeader<T::AccountId>>;
		/// Map of imported header hashes by number.
		EthHeadersByNumber: map hasher(blake2_128_concat) u64 => Option<Vec<H256>>;
	}

	add_extra_genesis {
		config(initial_header): EthereumHeader;
		config(initial_difficulty): U256;

		build(|config| {
			let initial_header = &config.initial_header;
			let initial_hash = initial_header.compute_hash();

			EthBestBlock::put((
				EthereumHeaderId {
					number: initial_header.number,
					hash: initial_hash,
				},
				config.initial_difficulty,
			));
			EthHeaders::<T>::insert(
				initial_hash,
				StoredEthereumHeader {
					submitter: None,
					header: initial_header.clone(),
					total_difficulty: config.initial_difficulty,
				},
			);
			EthHeadersByNumber::insert(
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
		/// Verification scheme is not supported.
		NotSupported,
		/// The message failed verification.
		Invalid
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn import_header(origin, header: EthereumHeader) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			Self::validate_header_to_import(&sender, &header)?;
			Self::import_validated_header(&sender, &header)
		}
	}
}

impl<T: Trait> Module<T> {

	/// Verify a message
	fn do_verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		Self::verify_sender(&sender)?;

		// Hash all inputs together to produce a unique key for the message
		let (block_no, event_idx) = match message.verification {
			VerificationInput::Basic { block_number, event_index } => (block_number, event_index),
			VerificationInput::None => return Err(Error::<T>::NotSupported.into())
		};
		let key_input = (app_id, message.payload.clone(), block_no, event_idx);
		let key = T::Hashing::hash_of(&key_input);

		// Verify that the message has not been seen before (replay protection)
		if <VerifiedPayloads<T>>::contains_key(key) {
			return Err(Error::<T>::Invalid.into())
		} else {
			<VerifiedPayloads<T>>::insert(key, ());
		}

		Ok(())
	}

	// Verify that the message sender matches the relayer account
	fn verify_sender(sender: &T::AccountId) -> DispatchResult {
		if *sender != RelayKey::<T>::get() {
			return Err(Error::<T>::Invalid.into())
		}
		Ok(())
	}

	// Validate an Ethereum header for import
	fn validate_header_to_import(sender: &T::AccountId, header: &EthereumHeader) -> Result<(), &'static str> {
		Self::verify_sender(sender)?;

		ensure!(
			EthHeaders::<T>::contains_key(header.parent_hash),
			"Parent header must be imported first",
		);

		let hash = header.compute_hash();
		ensure!(
			!EthHeaders::<T>::contains_key(hash),
			"Header can only be imported once",
		);

		// TODO check PoW

		Ok(())
	}

	// Import a new, validated Ethereum header
	fn import_validated_header(sender: &T::AccountId, header: &EthereumHeader) -> DispatchResult {
		let hash = header.compute_hash();
		let stored_parent_header = EthHeaders::<T>::get(header.parent_hash).unwrap();
		let total_difficulty = stored_parent_header.total_difficulty
			.checked_add(header.difficulty)
			.ok_or("Total difficulty overflow")?;
		let header_to_store = StoredEthereumHeader {
			submitter: Some(sender.clone()),
			header: header.clone(),
			total_difficulty,
		};

		EthHeaders::<T>::insert(hash, header_to_store);

		if EthHeadersByNumber::contains_key(header.number) {
			EthHeadersByNumber::mutate(header.number, |option| {
				let hashes = option.as_mut().unwrap();
				hashes.push(hash);
			});
		} else {
			EthHeadersByNumber::insert(header.number, vec![hash]);
		}

		// Maybe track new highest difficulty chain
		let (_, highest_difficulty) = EthBestBlock::get();
		if total_difficulty > highest_difficulty {
			EthBestBlock::put((
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

impl<T: Trait> Verifier<T::AccountId> for Module<T> {
	fn verify(sender: T::AccountId, app_id: AppId, message: &Message) -> DispatchResult {
		Self::do_verify(sender, app_id, message)?;
		Ok(())
	}
}
