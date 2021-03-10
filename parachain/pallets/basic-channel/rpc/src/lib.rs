use itertools::zip;

use codec::Codec;
use jsonrpc_core::{Result, Error as JsonError};
use jsonrpc_derive::rpc;
use sp_core::H256;
use sp_runtime::offchain::storage::StorageValueRef;

use artemis_basic_channel::outbound::{CommitmentData, generate_merkle_proofs, offchain_key};

type Proofs<AccountId> = Vec<(AccountId, Vec<u8>)>;

#[rpc]
pub trait BasicChannelApi<T>
{
	#[rpc(name = "get_merkle_proofs")]
	fn get_merkle_proofs(&self, root: H256) -> Result<Proofs<T>>;
}

pub struct BasicChannel<AccountId> {
	indexing_prefix: &'static [u8],
	_marker: std::marker::PhantomData<AccountId>
}

impl<AccountId> BasicChannel<AccountId> {
	pub fn new(indexing_prefix: &'static [u8]) -> Self {
		Self {
			indexing_prefix,
			_marker: Default::default(),
		}
	}
}

impl<AccountId> BasicChannelApi<AccountId> for BasicChannel<AccountId>
where
	AccountId: Codec + Send + Sync + 'static,
{
	fn get_merkle_proofs(&self, root: H256) -> Result<Proofs<AccountId>> {
		let key = offchain_key(self.indexing_prefix, root);
		let data = StorageValueRef::persistent(&key);

		if let Some(Some(cdata)) = data.get::<CommitmentData<AccountId>>() {
			let num_coms = cdata.subcommitments.len();
			let mut accounts = Vec::with_capacity(num_coms);
			let mut commitments = Vec::with_capacity(num_coms);

			for (acc, com) in cdata.subcommitments {
				accounts.push(acc);
				commitments.push(com);
			};

			match generate_merkle_proofs(commitments.into_iter()) {
				Ok(proofs) => Ok(zip(accounts, proofs).collect::<Proofs<AccountId>>()),
				Err(_) => Err(JsonError::invalid_request()),
			}
		} else {
			Err(JsonError::invalid_request())
                }
	}
}
