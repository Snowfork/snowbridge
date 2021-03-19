use std::sync::Arc;

use itertools::zip;
use parking_lot::RwLock;

use codec::{Codec, Decode, Encode};
use jsonrpc_core::{Result, Error as JsonError};
use jsonrpc_derive::rpc;
use sc_rpc::DenyUnsafe;
use sp_core::{Bytes, offchain::OffchainStorage};

use artemis_basic_channel::outbound::{CommitmentData, generate_merkle_proofs};

#[cfg(test)]
mod tests;

type Proofs<TAccountId> = Vec<(TAccountId, Vec<u8>)>;

#[rpc]
pub trait BasicChannelApi<TAccountId>
{
	#[rpc(name = "basicChannel_getMerkleProofs")]
	fn get_merkle_proofs(&self, key: Bytes) -> Result<Bytes>;
}

pub struct BasicChannel<TStorage: OffchainStorage, TAccountId> {
	_marker: std::marker::PhantomData<TAccountId>,
	/// Offchain storage
	storage: Arc<RwLock<TStorage>>,
	/// Standard Substrate RPC check
	deny_unsafe: DenyUnsafe,
}

impl<TStorage, TAccountId> BasicChannel<TStorage, TAccountId>
where
	TStorage: OffchainStorage,
{
	pub fn new(storage: TStorage, deny_unsafe: DenyUnsafe) -> Self {
		Self {
			_marker: Default::default(),
			deny_unsafe,
			storage: Arc::new(RwLock::new(storage)),
		}
	}
}

impl<TStorage, TAccountId> BasicChannelApi<TAccountId> for BasicChannel<TStorage, TAccountId>
where
	TAccountId: Codec + Send + Sync + 'static,
	TStorage: OffchainStorage + Send + Sync + 'static,
{
	fn get_merkle_proofs(&self, key: Bytes) -> Result<Bytes> {
		self.deny_unsafe.check_if_safe()?;

		// For some reason, the TestPersistentOffchainDB used for testing this, removes the prefixes
		// when persisting the offchain overlay.
		#[cfg(test)]
		let prefix = b"";
		#[cfg(not(test))]
		let prefix = &sp_core::offchain::STORAGE_PREFIX;

		if let Some(data) = self.storage.read().get(prefix, &*key) {
			if let Ok(cdata) = <CommitmentData<TAccountId>>::decode(&mut data.as_slice()) {
				let num_coms = cdata.subcommitments.len();
				let mut accounts = Vec::with_capacity(num_coms);
				let mut commitments = Vec::with_capacity(num_coms);
				cdata.subcommitments.into_iter().for_each(|s| {
					accounts.push(s.account_id);
					commitments.push(s.flat_commitment);
				});
				match generate_merkle_proofs(commitments.into_iter()) {
					Ok(proofs) => {
						let pairs = zip(accounts, proofs).collect::<Proofs<TAccountId>>();
						Ok(Bytes::from(Encode::encode(&pairs)))
					}
					Err(_) => Err(JsonError::invalid_request()),
				}
			} else {
				Err(JsonError::internal_error())
			}
		} else {
			Err(JsonError::invalid_params("Key not found"))
		}
	}
}
