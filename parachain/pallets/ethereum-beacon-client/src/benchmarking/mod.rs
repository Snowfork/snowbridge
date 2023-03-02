use super::*;

use crate::Pallet as EthereumBeaconClient;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

#[cfg(feature = "minimal")]
mod data_minimal;
#[cfg(feature = "minimal")]
use data_minimal::*;

#[cfg(not(feature = "minimal"))]
mod data_mainnet;
#[cfg(not(feature = "minimal"))]
use data_mainnet::*;

benchmarks! {
	sync_committee_period_update {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = initial_sync();

		EthereumBeaconClient::<T>::initial_sync(initial_sync_data.clone())?;

		let sync_committee_update = sync_committee_update();

		//initialize SyncCommittees with period in sync_committee_update
		LatestSyncCommitteePeriod::<T>::set(EthereumBeaconClient::<T>::compute_current_sync_period(
				sync_committee_update.attested_header.slot,
			));
		SyncCommittees::<T>::insert(
			EthereumBeaconClient::<T>::compute_current_sync_period(
				sync_committee_update.attested_header.slot,
			),
			initial_sync_data.current_sync_committee,
		);

	}: sync_committee_period_update(RawOrigin::Signed(caller.clone()), sync_committee_update.clone())
	verify {
		assert!(<SyncCommittees<T>>::get(sync_committee_update.sync_committee_period+1).pubkeys.len() > 0);
	}

	import_finalized_header {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = initial_sync();

		EthereumBeaconClient::<T>::initial_sync(initial_sync_data.clone())?;

		let finalized_header_update = finalized_header_update();

		SyncCommittees::<T>::insert(
			EthereumBeaconClient::<T>::compute_current_sync_period(
				finalized_header_update.attested_header.slot,
			),
			initial_sync_data.current_sync_committee,
		);

		//initialize LatestFinalizedHeaderState with parent slot of finalized_header_update
		LatestFinalizedHeaderState::<T>::set(FinalizedHeaderState {
			beacon_block_root: Default::default(),
			import_time: initial_sync_data.import_time + 51200,
			beacon_slot: finalized_header_update.finalized_header.slot - 1,
		});

	}: _(RawOrigin::Signed(caller.clone()), finalized_header_update.clone())
	verify {
		let header_hash_bytes = merkleization::hash_tree_root_beacon_header(finalized_header_update.finalized_header).unwrap();

		let header_hash: H256 = header_hash_bytes.into();

		<FinalizedBeaconHeaders<T>>::get(header_hash).unwrap();
	}

	import_execution_header {
		let caller: T::AccountId = whitelisted_caller();

		let initial_sync_data = initial_sync();

		EthereumBeaconClient::<T>::initial_sync(initial_sync_data.clone())?;

		let block_update = block_update();

		SyncCommittees::<T>::insert(EthereumBeaconClient::<T>::compute_current_sync_period(
				block_update.block.slot,
			), initial_sync_data.current_sync_committee);

		let finalized_update: FinalizedHeaderUpdate<T::MaxSignatureSize, T::MaxProofBranchSize, T::MaxSyncCommitteeSize> = finalized_header_update();

		let finalized_slot = finalized_update.finalized_header.slot;
		let finalized_block_root: H256 =
			merkleization::hash_tree_root_beacon_header(finalized_update.finalized_header)
				.unwrap()
				.into();

		LatestFinalizedHeaderState::<T>::set(FinalizedHeaderState{
			beacon_block_root: finalized_block_root,
			beacon_slot: finalized_slot,
			import_time: 0,
		});
		FinalizedBeaconHeadersBlockRoot::<T>::insert(
			finalized_block_root,
			finalized_update.block_roots_hash,
		);
	}: _(RawOrigin::Signed(caller.clone()), block_update.clone())
	verify {
		let block_hash: H256 = block_update.block.body.execution_payload.block_hash;

		<ExecutionHeaders<T>>::get(block_hash).unwrap();
	}
}

#[cfg(feature = "minimal")]
impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::new_tester::<crate::mock::mock_minimal::Test>(),
	crate::mock::mock_minimal::Test,
);

#[cfg(not(feature = "minimal"))]
impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::new_tester::<crate::mock::mock_mainnet::Test>(),
	crate::mock::mock_mainnet::Test,
);
