//! EthereumLightClient pallet benchmarking
use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

#[allow(unused_imports)]
use crate::Pallet as EthereumLightClient;

mod data;

/// The index up until which headers are reserved for pruning. The header at
/// `data::headers_11963025_to_11963069()[RESERVED_FOR_PRUNING]` is specially
/// chosen to be a sibling of the previous header. Indices 0 to RESERVED_FOR_PRUNING - 1
/// contain strictly increasing block numbers.
const RESERVED_FOR_PRUNING: usize = HEADERS_TO_PRUNE_IN_SINGLE_IMPORT as usize;

fn get_best_block<T: Config>() -> (EthereumHeaderId, U256) {
	<BestBlock<T>>::get()
}

fn get_blocks_to_prune<T: Config>() -> PruningRange {
	<BlocksToPrune<T>>::get()
}

fn set_blocks_to_prune<T: Config>(oldest_unpruned: u64, oldest_to_keep: u64) {
	<BlocksToPrune<T>>::put(PruningRange {
		oldest_unpruned_block: oldest_unpruned,
		oldest_block_to_keep: oldest_to_keep,
	});
}

fn assert_header_pruned<T: Config>(hash: H256, number: u64) {
	assert!(Headers::<T>::get(hash).is_none());

	let hashes_at_number = <HeadersByNumber<T>>::get(number);
	assert!(hashes_at_number.is_none() || !hashes_at_number.unwrap().contains(&hash),);
}

// NOTE: These benchmarks only run successully using the `snowbridge' runtime, which is configured
// for Ethereum mainnet.

benchmarks! {
	// Benchmark `import_header` extrinsic under worst case conditions:
	// * Import will set a new best block.
	// * Import will set a new finalized header.
	// * Import will iterate over the max value of DescendantsUntilFinalized headers
	//   in the chain.
	// * Import will prune HEADERS_TO_PRUNE_IN_SINGLE_IMPORT headers.
	// * Pruned headers will come from distinct block numbers so that we have the max
	//   number of HeaderByNumber::take calls.
	// * The last pruned header will have siblings that we don't prune and have to
	//   re-insert using <HeadersByNumber<T>>::insert.
	import_header {
		let caller: T::AccountId = whitelisted_caller();
		let descendants_until_final = T::DescendantsUntilFinalized::get();

		let next_finalized_idx = RESERVED_FOR_PRUNING + 1;
		let next_tip_idx = next_finalized_idx + descendants_until_final as usize;
		let headers = data::headers_11963025_to_11963069();
		let header = headers[next_tip_idx].clone();
		let header_proof = data::header_proof(header.compute_hash()).unwrap();

		EthereumLightClient::<T>::initialize_storage(
			headers[0..next_tip_idx].to_vec(),
			U256::zero(),
			descendants_until_final,
		)?;

		set_blocks_to_prune::<T>(
			headers[0].number,
			headers[next_finalized_idx].number,
		);

	}: _(RawOrigin::Signed(caller.clone()), header, header_proof)
	verify {
		// Check that the best header has been updated
		let best = &headers[next_tip_idx];
		assert_eq!(
			get_best_block::<T>().0,
			EthereumHeaderId {
				number: best.number,
				hash: best.compute_hash(),
			},
		);

		// Check that `RESERVED_FOR_PRUNING` headers have been pruned
		// while leaving 1 sibling behind
		headers[0..RESERVED_FOR_PRUNING]
			.iter()
			.for_each(|h| assert_header_pruned::<T>(h.compute_hash(), h.number));
		let last_pruned_sibling = &headers[RESERVED_FOR_PRUNING];
		assert_eq!(
			get_blocks_to_prune::<T>().oldest_unpruned_block,
			last_pruned_sibling.number,
		);
	}

	// Benchmark `import_header` extrinsic under worst case conditions:
	// * Import will set a new best block.
	// * Import will *not* set a new finalized header because its sibling was imported first.
	// * Import will iterate over the max value of DescendantsUntilFinalized headers
	//   in the chain.
	// * Import will prune HEADERS_TO_PRUNE_IN_SINGLE_IMPORT headers.
	// * Pruned headers will come from distinct block numbers so that we have the max
	//   number of HeaderByNumber::take calls.
	// * The last pruned header will have siblings that we don't prune and have to
	//   re-insert using <HeadersByNumber<T>>::insert.
	import_header_not_new_finalized_with_max_prune {
		let caller: T::AccountId = whitelisted_caller();
		let descendants_until_final = T::DescendantsUntilFinalized::get();

		let finalized_idx = RESERVED_FOR_PRUNING + 1;
		let next_tip_idx = finalized_idx + descendants_until_final as usize;
		let headers = data::headers_11963025_to_11963069();
		let header = headers[next_tip_idx].clone();
		let header_proof = data::header_proof(header.compute_hash()).unwrap();

		let mut header_sibling = header.clone();
		header_sibling.difficulty -= 1.into();
		let mut init_headers = headers[0..next_tip_idx].to_vec();
		init_headers.append(&mut vec![header_sibling]);

		EthereumLightClient::<T>::initialize_storage(
			init_headers,
			U256::zero(),
			descendants_until_final,
		)?;

		set_blocks_to_prune::<T>(
			headers[0].number,
			headers[finalized_idx].number,
		);

	}: import_header(RawOrigin::Signed(caller.clone()), header, header_proof)
	verify {
		// Check that the best header has been updated
		let best = &headers[next_tip_idx];
		assert_eq!(
			get_best_block::<T>().0,
			EthereumHeaderId {
				number: best.number,
				hash: best.compute_hash(),
			},
		);

		// Check that `RESERVED_FOR_PRUNING` headers have been pruned
		// while leaving 1 sibling behind
		headers[0..RESERVED_FOR_PRUNING]
			.iter()
			.for_each(|h| assert_header_pruned::<T>(h.compute_hash(), h.number));
		let last_pruned_sibling = &headers[RESERVED_FOR_PRUNING];
		assert_eq!(
			get_blocks_to_prune::<T>().oldest_unpruned_block,
			last_pruned_sibling.number,
		);
	}

	// Benchmark `import_header` extrinsic under average case conditions:
	// * Import will set a new best block.
	// * Import will set a new finalized header.
	// * Import will iterate over the max value of DescendantsUntilFinalized headers
	//   in the chain.
	// * Import will prune a single old header with no siblings.
	import_header_new_finalized_with_single_prune {
		let caller: T::AccountId = whitelisted_caller();
		let descendants_until_final = T::DescendantsUntilFinalized::get();

		let finalized_idx = RESERVED_FOR_PRUNING + 1;
		let next_tip_idx = finalized_idx + descendants_until_final as usize;
		let headers = data::headers_11963025_to_11963069();
		let header = headers[next_tip_idx].clone();
		let header_proof = data::header_proof(header.compute_hash()).unwrap();

		EthereumLightClient::<T>::initialize_storage(
			headers[0..next_tip_idx].to_vec(),
			U256::zero(),
			descendants_until_final,
		)?;

		set_blocks_to_prune::<T>(
			headers[0].number,
			headers[0].number + 1,
		);

	}: import_header(RawOrigin::Signed(caller.clone()), header, header_proof)
	verify {
		// Check that the best header has been updated
		let best = &headers[next_tip_idx];
		assert_eq!(
			get_best_block::<T>().0,
			EthereumHeaderId {
				number: best.number,
				hash: best.compute_hash(),
			},
		);

		// Check that 1 header has been pruned
		let oldest_header = &headers[0];
		assert_header_pruned::<T>(oldest_header.compute_hash(), oldest_header.number);
		assert_eq!(
			get_blocks_to_prune::<T>().oldest_unpruned_block,
			oldest_header.number + 1,
		);
	}

	// Benchmark `import_header` extrinsic under average case conditions:
	// * Import will set a new best block.
	// * Import will *not* set a new finalized header because its sibling was imported first.
	// * Import will iterate over the max value of DescendantsUntilFinalized headers
	//   in the chain.
	// * Import will prune a single old header with no siblings.
	import_header_not_new_finalized_with_single_prune {
		let caller: T::AccountId = whitelisted_caller();
		let descendants_until_final = T::DescendantsUntilFinalized::get();

		let finalized_idx = RESERVED_FOR_PRUNING + 1;
		let next_tip_idx = finalized_idx + descendants_until_final as usize;
		let headers = data::headers_11963025_to_11963069();
		let header = headers[next_tip_idx].clone();
		let header_proof = data::header_proof(header.compute_hash()).unwrap();

		let mut header_sibling = header.clone();
		header_sibling.difficulty -= 1.into();
		let mut init_headers = headers[0..next_tip_idx].to_vec();
		init_headers.append(&mut vec![header_sibling]);

		EthereumLightClient::<T>::initialize_storage(
			init_headers,
			U256::zero(),
			descendants_until_final,
		)?;

		set_blocks_to_prune::<T>(
			headers[0].number,
			headers[0].number + 1,
		);

	}: import_header(RawOrigin::Signed(caller.clone()), header, header_proof)
	verify {
		// Check that the best header has been updated
		let best = &headers[next_tip_idx];
		assert_eq!(
			get_best_block::<T>().0,
			EthereumHeaderId {
				number: best.number,
				hash: best.compute_hash(),
			},
		);

		// Check that 1 header has been pruned
		let oldest_header = &headers[0];
		assert_header_pruned::<T>(oldest_header.compute_hash(), oldest_header.number);
		assert_eq!(
			get_blocks_to_prune::<T>().oldest_unpruned_block,
			oldest_header.number + 1,
		);
	}
}

impl_benchmark_test_suite!(
	EthereumLightClient,
	crate::mock::new_tester::<crate::mock::mock_verifier_with_pow::Test>(),
	crate::mock::mock_verifier_with_pow::Test,
);
