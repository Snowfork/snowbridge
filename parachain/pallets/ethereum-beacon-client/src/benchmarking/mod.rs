use super::*;

use crate::Pallet as EthereumBeaconClient;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

// For benchmark focus on main spec only
mod data_mainnet;
use data_mainnet::*;

mod util;
use primitives::{
	fast_aggregate_verify, fast_aggregate_verify_legacy, prepare_aggregate_pubkey,
	prepare_aggregate_signature,
};
use util::*;

benchmarks! {
	sync_committee_period_update {
		let caller: T::AccountId = whitelisted_caller();

		let sync_committee_update = initialize_sync_committee::<T>()?;

	}: sync_committee_period_update(RawOrigin::Signed(caller.clone()), sync_committee_update.clone())
	verify {
		EthereumBeaconClient::<T>::sync_committee_for_period(sync_committee_update.sync_committee_period+1).unwrap();
	}

	import_finalized_header {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = initial_sync();

		EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

		let finalized_header_update = finalized_header_update();

		let current_period = EthereumBeaconClient::<T>::compute_current_sync_period(
				finalized_header_update.attested_header.slot,
			);

		EthereumBeaconClient::<T>::store_sync_committee(current_period, &initial_sync_data.current_sync_committee)?;

		//initialize LatestFinalizedHeaderState with parent slot of finalized_header_update
		LatestFinalizedHeaderState::<T>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			import_time: initial_sync_data.import_time + 51200,
			beacon_slot: finalized_header_update.finalized_header.slot - 1,
		});

	}: _(RawOrigin::Signed(caller.clone()), finalized_header_update.clone())
	verify {
		let header_hash: H256 = finalized_header_update.finalized_header.hash_tree_root().unwrap();

		<FinalizedBeaconHeaders<T>>::get(header_hash).unwrap();
	}

	import_execution_header {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = initial_sync();

		EthereumBeaconClient::<T>::process_checkpoint_update(&initial_sync_data)?;

		let header_update = header_update();

		let current_period = EthereumBeaconClient::<T>::compute_current_sync_period(
				header_update.beacon_header.slot,
			);

		EthereumBeaconClient::<T>::store_sync_committee(current_period, &initial_sync_data.current_sync_committee)?;

		let finalized_update: FinalizedHeaderUpdate<> = finalized_header_update();

		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root = finalized_update.finalized_header.hash_tree_root()
				.unwrap();

		LatestFinalizedHeaderState::<T>::set(FinalizedHeaderState{
			beacon_block_root: finalized_block_root,
			beacon_slot: finalized_slot,
			import_time: 0,
		});
		FinalizedBeaconHeadersBlockRoot::<T>::insert(
			finalized_block_root,
			finalized_update.block_roots_root,
		);
	}: _(RawOrigin::Signed(caller.clone()), header_update.clone())
	verify {
		assert!(<ExecutionHeaders<T>>::contains_key(header_update.execution_header.block_hash))
	}

	activate_bridge {
	}: _(RawOrigin::Root)
	verify {
		assert!(!<Blocked<T>>::get());
	}

	deactivate_bridge {
	}: _(RawOrigin::Root)
	verify {
		assert!(<Blocked<T>>::get());
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

	bls_fast_aggregate_verify_legacy {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = participant_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
	}:{
		fast_aggregate_verify_legacy(&participant_pubkeys, signing_root, &update.sync_aggregate.sync_committee_signature).unwrap();
	}

	bls_fast_aggregate_verify {
		let update = initialize_sync_committee::<T>()?;
		let current_sync_committee = sync_committee::<T>(&update)?;
		let absent_pubkeys = absent_pubkeys::<T>(&update)?;
		let signing_root = signing_root::<T>(&update)?;
	}:{
		fast_aggregate_verify(&current_sync_committee.aggregate_pubkey, &absent_pubkeys, signing_root, &update.sync_aggregate.sync_committee_signature).unwrap();
	}
}

impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::new_tester::<crate::mock::mock_mainnet::Test>(),
	crate::mock::mock_mainnet::Test,
);
