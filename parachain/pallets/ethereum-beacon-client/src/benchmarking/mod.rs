use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use crate::Pallet as EthereumBeaconClient;

mod data;

benchmarks! {
	sync_committee_period_update {
		let caller: T::AccountId = whitelisted_caller();

		EthereumBeaconClient::<T>::initial_sync(data::initial_sync()).unwrap();

		let other_sync_committee_period_update = data::sync_committee_update();
        
    }: sync_committee_period_update(RawOrigin::Signed(caller.clone()), other_sync_committee_period_update)
    verify {
        assert!(<SyncCommittees<T>>::get(2).pubkeys.len() > 0);
    }

	import_finalized_header {
		let caller: T::AccountId = whitelisted_caller();

		EthereumBeaconClient::<T>::initial_sync(data::initial_sync()).unwrap();

		let finalized_header = data::finalized_header_update();

	}: _(RawOrigin::Signed(caller.clone()), finalized_header.clone())
	verify {
		let header_hash_bytes = merkleization::hash_tree_root_beacon_header(finalized_header.finalized_header).unwrap();

		let header_hash: H256 = header_hash_bytes.into();

        <FinalizedBeaconHeaders<T>>::get(header_hash).unwrap();
    }

	import_execution_header {
		let caller: T::AccountId = whitelisted_caller();

		EthereumBeaconClient::<T>::initial_sync(data::initial_sync()).unwrap();

		let block_update = data::block_update();
	}: _(RawOrigin::Signed(caller.clone()), block_update.clone())
	verify {
		let block_hash: H256 = block_update.block.body.execution_payload.block_hash;

        <ExecutionHeaders<T>>::get(block_hash).unwrap();
    }
}

impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::test::new_tester(),
	crate::test::Test,
);
