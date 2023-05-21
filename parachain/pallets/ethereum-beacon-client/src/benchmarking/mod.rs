use super::*;

mod fixtures;
mod util;

use crate::Pallet as EthereumBeaconClient;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

use fixtures::{make_checkpoint, make_execution_header_update, make_finalized_header_update};

use primitives::{
	fast_aggregate_verify, prepare_aggregate_pubkey, prepare_aggregate_signature,
	verify_merkle_branch, CompactBeaconState,
};
use util::*;

benchmarks! {
	submit {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = make_checkpoint();

		EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

		let finalized_header_update = make_finalized_header_update();

		let current_period = compute_period(
				finalized_header_update.attested_header.slot,
			);

		//EthereumBeaconClient::<T>::store_sync_committee(current_period, &initial_sync_data.current_sync_committee)?;

		// initialize LatestFinalizedHeaderState with parent slot of finalized_header_update
		LatestFinalizedHeader::<T>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			beacon_slot: finalized_header_update.finalized_header.slot - 1,
		});

	}: submit(RawOrigin::Signed(caller.clone()), finalized_header_update.clone())
	verify {
		let header_hash: H256 = finalized_header_update.finalized_header.hash_tree_root().unwrap();

		<FinalizedBeaconState<T>>::get(header_hash).unwrap();
	}

	submit_with_sync_committee {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = make_checkpoint();
		let sync_committee_update = initialize_sync_committee::<T>()?;

		let period = compute_period(sync_committee_update.attested_header.slot);

		// initialize LatestFinalizedHeaderState with parent slot of finalized_header_update
		LatestFinalizedHeader::<T>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			beacon_slot: sync_committee_update.finalized_header.slot - 1,
		});

	}: submit(RawOrigin::Signed(caller.clone()), sync_committee_update.clone())
	verify {
		assert!(<NextSyncCommittee<T>>::exists())
	}

	submit_execution_header {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = make_checkpoint();

		EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

		let header_update = make_execution_header_update();

		let current_period = compute_period(
				header_update.header.slot,
			);

		//EthereumBeaconClient::<T>::store_sync_committee(current_period, &initial_sync_data.current_sync_committee)?;

		let finalized_update = make_finalized_header_update();

		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root = finalized_update.finalized_header.hash_tree_root()
				.unwrap();

		LatestFinalizedHeader::<T>::set(FinalizedHeaderState{
			beacon_block_root: finalized_block_root,
			beacon_slot: finalized_slot,
		});
		FinalizedBeaconState::<T>::insert(
			finalized_block_root,
			CompactBeaconState {
				slot: finalized_update.finalized_header.slot,
				block_roots_root: finalized_update.block_roots_root,
			}
		);
	}: _(RawOrigin::Signed(caller.clone()), header_update.clone())
	verify {
		assert!(<ExecutionHeaders<T>>::contains_key(header_update.execution_header.block_hash))
	}

	bls_fast_aggregate_verify_pre_aggregated {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = participant_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
		let agg_sig = prepare_aggregate_signature(&update.sync_aggregate.sync_committee_signature).unwrap();
		let agg_pub_key = prepare_aggregate_pubkey(&participant_pubkeys).unwrap();
	}:{
		agg_sig.fast_aggregate_verify_pre_aggregated(signing_root.as_bytes(), &agg_pub_key)
	}

	bls_fast_aggregate_verify {
		let update = initialize_sync_committee::<T>()?;
		let current_sync_committee = <CurrentSyncCommittee<T>>::get();
		let absent_pubkeys = absent_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
	}:{
		fast_aggregate_verify(&current_sync_committee.aggregate_pubkey, &absent_pubkeys, signing_root, &update.sync_aggregate.sync_committee_signature).unwrap();
	}

	merkle_branch_verify {
		let update = initialize_sync_committee::<T>()?;
		let block_root: H256 = update.finalized_header.hash_tree_root().unwrap();
	}:{
		verify_merkle_branch(block_root,&update.finality_branch,config::FINALIZED_ROOT_SUBTREE_INDEX,
					config::FINALIZED_ROOT_DEPTH,update.attested_header.state_root);
	}
}

impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::mainnet::new_tester(),
	crate::mock::mainnet::Test
);
