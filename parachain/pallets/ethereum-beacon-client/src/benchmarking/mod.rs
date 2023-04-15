use super::*;

use crate::Pallet as EthereumBeaconClient;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

// For benchmark focus on main spec only
mod data_mainnet;
use data_mainnet::*;
mod util;
use util::*;

benchmarks! {
	sync_committee_period_update {
		let caller: T::AccountId = whitelisted_caller();

		let sync_committee_update = initialize_sync_committee::<T>()?;

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

		let header_update = header_update();

		SyncCommittees::<T>::insert(EthereumBeaconClient::<T>::compute_current_sync_period(
				header_update.beacon_header.slot,
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
			finalized_update.block_roots_root,
		);
	}: _(RawOrigin::Signed(caller.clone()), header_update.clone())
	verify {
		let header: ExecutionHeader = header_update.execution_header.try_into().unwrap();
		<ExecutionHeaders<T>>::get(header.block_hash).unwrap();
	}

	unblock_bridge {
	}: _(RawOrigin::Root)
	verify {
		assert_eq!(<Blocked<T>>::get(),false);
	}

	bls_fast_aggregate_verify {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = get_participant_pubkeys::<T>(&update)?;
		let signing_root = get_signing_message::<T>(&update)?;
	}:{
		EthereumBeaconClient::<T>::bls_fast_aggregate_verify(participant_pubkeys,signing_root,update.sync_aggregate.sync_committee_signature)?;
	}

	bls_aggregate_pubkey {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = get_participant_pubkeys::<T>(&update)?;
	}:{
		participant_pubkeys
				.iter()
				.map(|bytes| milagro_bls::PublicKey::from_bytes_unchecked(&bytes.0))
				.collect::<Result<Vec<milagro_bls::PublicKey>, _>>().unwrap()
	}

	bls_verify_message {
		let update = initialize_sync_committee::<T>()?;
		let participant_pubkeys = get_participant_pubkeys::<T>(&update)?;
		let signing_root = get_signing_message::<T>(&update)?;
		let agg_sig = get_aggregate_signature::<T>(update.sync_aggregate.sync_committee_signature).unwrap();
		let agg_pub_key = get_aggregate_pubkey::<T>(participant_pubkeys).unwrap();
	}:{
		agg_sig.fast_aggregate_verify_pre_aggregated(&signing_root.as_bytes(), &agg_pub_key)
	}
}

impl_benchmark_test_suite!(
	EthereumBeaconClient,
	crate::mock::new_tester::<crate::mock::mock_mainnet::Test>(),
	crate::mock::mock_mainnet::Test,
);
