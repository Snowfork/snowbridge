use super::*;

mod fixtures;
mod util;

use crate::Pallet as EthereumBeaconClient;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

use fixtures::{
	make_checkpoint, make_execution_header_update, make_finalized_header_update,
	make_sync_committee_update,
};

use primitives::{
	fast_aggregate_verify, prepare_aggregate_pubkey, prepare_aggregate_signature,
	verify_merkle_branch,
};
use util::*;

benchmarks! {
	force_checkpoint {
		let caller: T::AccountId = whitelisted_caller();
		let checkpoint_update = make_checkpoint();

	}: _(RawOrigin::Root, checkpoint_update.clone())
	verify {
		let block_root: H256 = checkpoint_update.header.hash_tree_root().unwrap();
		assert!(<LatestFinalizedBlockRoot<T>>::get() == block_root);
		assert!(<FinalizedBeaconState<T>>::get(block_root).is_some());
	}

	submit {
		let caller: T::AccountId = whitelisted_caller();
		let checkpoint_update = make_checkpoint();
		let finalized_header_update = make_finalized_header_update();
		EthereumBeaconClient::<T>::process_checkpoint_update(&checkpoint_update)?;

	}: submit(RawOrigin::Signed(caller.clone()), finalized_header_update.clone())
	verify {
		let block_root: H256 = finalized_header_update.finalized_header.hash_tree_root().unwrap();
		assert!(<LatestFinalizedBlockRoot<T>>::get() == block_root);
		assert!(<FinalizedBeaconState<T>>::get(block_root).is_some());
	}

	submit_with_sync_committee {
		let caller: T::AccountId = whitelisted_caller();
		let checkpoint_update = make_checkpoint();
		let sync_committee_update = make_sync_committee_update();
		EthereumBeaconClient::<T>::process_checkpoint_update(&checkpoint_update)?;

	}: submit(RawOrigin::Signed(caller.clone()), sync_committee_update.clone())
	verify {
		assert!(<NextSyncCommittee<T>>::exists())
	}

	submit_execution_header {
		let caller: T::AccountId = whitelisted_caller();
		let checkpoint_update = make_checkpoint();
		let finalized_header_update = make_finalized_header_update();
		let execution_header_update = make_execution_header_update();
		EthereumBeaconClient::<T>::process_checkpoint_update(&checkpoint_update)?;
		EthereumBeaconClient::<T>::process_update(&finalized_header_update)?;
	}: _(RawOrigin::Signed(caller.clone()), execution_header_update.clone())
	verify {
		assert!(<ExecutionHeaders<T>>::contains_key(execution_header_update.execution_header.block_hash))
	}

	#[extra]
	bls_fast_aggregate_verify_pre_aggregated {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = participant_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
		let agg_sig = prepare_aggregate_signature(&update.sync_aggregate.sync_committee_signature).unwrap();
		let agg_pub_key = prepare_aggregate_pubkey(&participant_pubkeys).unwrap();
	}:{
		agg_sig.fast_aggregate_verify_pre_aggregated(signing_root.as_bytes(), &agg_pub_key)
	}

	#[extra]
	bls_fast_aggregate_verify {
		let update = initialize_sync_committee::<T>()?;
		let current_sync_committee = <CurrentSyncCommittee<T>>::get();
		let absent_pubkeys = absent_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
	}:{
		fast_aggregate_verify(&current_sync_committee.aggregate_pubkey, &absent_pubkeys, signing_root, &update.sync_aggregate.sync_committee_signature).unwrap();
	}

	#[extra]
	merkle_branch_verify {
		let update = initialize_sync_committee::<T>()?;
		let block_root: H256 = update.finalized_header.hash_tree_root().unwrap();
	}:{
		verify_merkle_branch(
			block_root,
			&update.finality_branch,
			config::FINALIZED_ROOT_SUBTREE_INDEX,
			config::FINALIZED_ROOT_DEPTH,update.attested_header.state_root
		);
	}
}

impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::mainnet::new_tester(),
	crate::mock::mainnet::Test
);
