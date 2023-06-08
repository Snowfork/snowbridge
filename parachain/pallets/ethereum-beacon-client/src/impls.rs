use super::*;

use frame_support::dispatch::DispatchError;
use snowbridge_ethereum::{Log, Receipt};

impl<T: Config> Verifier for Pallet<T> {
	/// Verify a message by verifying the existence of the corresponding
	/// Ethereum log in a block. Returns the log if successful.
	fn verify(message: &Message) -> Result<Log, DispatchError> {
		log::info!(
			target: "ethereum-beacon-client",
			"💫 Verifying message with block hash {}",
			message.proof.block_hash,
		);

		let header = <ExecutionHeaderBuffer<T>>::get(message.proof.block_hash)
			.ok_or(Error::<T>::MissingHeader)?;

		let receipt = match Self::verify_receipt_inclusion(header.receipts_root, &message.proof) {
			Ok(receipt) => receipt,
			Err(err) => {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 Verification of receipt inclusion failed for block {}: {:?}",
					message.proof.block_hash,
					err
				);
				return Err(err)
			},
		};

		log::trace!(
			target: "ethereum-beacon-client",
			"💫 Verified receipt inclusion for transaction at index {} in block {}",
			message.proof.tx_index, message.proof.block_hash,
		);

		let log = match rlp::decode(&message.data) {
			Ok(log) => log,
			Err(err) => {
				log::error!(
					target: "ethereum-beacon-client",
					"💫 RLP log decoded failed {}: {:?}",
					message.proof.block_hash,
					err
				);
				return Err(Error::<T>::DecodeFailed.into())
			},
		};

		if !receipt.contains_log(&log) {
			log::error!(
				target: "ethereum-beacon-client",
				"💫 Event log not found in receipt for transaction at index {} in block {}",
				message.proof.tx_index, message.proof.block_hash,
			);
			return Err(Error::<T>::InvalidProof.into())
		}

		log::info!(
			target: "ethereum-beacon-client",
			"💫 Receipt verification successful for {}",
			message.proof.block_hash,
		);

		Ok(log)
	}
}

impl<T: Config> Pallet<T> {
	/// Verifies that the receipt encoded in proof.data is included in the block given by
	/// proof.block_hash. Inclusion is only recognized if the block has been finalized.
	pub fn verify_receipt_inclusion(
		receipts_root: H256,
		proof: &Proof,
	) -> Result<Receipt, DispatchError> {
		let result =
			verify_receipt_proof(receipts_root, &proof.data.1).ok_or(Error::<T>::InvalidProof)?;

		match result {
			Ok(receipt) => Ok(receipt),
			Err(err) => {
				log::trace!(
					target: "ethereum-beacon-client",
					"💫 Failed to decode transaction receipt: {}",
					err
				);
				Err(Error::<T>::InvalidProof.into())
			},
		}
	}
}
