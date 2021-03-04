//! VerifierLightclient pallet benchmarking

#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_system::RawOrigin;
use frame_benchmarking::{benchmarks, account, whitelisted_caller, impl_benchmark_test_suite};

use crate::Module as VerifierLightclient;

mod data;

fn get_best_block() -> (EthereumHeaderId, U256) {
	BestBlock::get()
}

fn get_finalized_block() -> EthereumHeaderId {
	FinalizedBlock::get()
}

fn get_header<T: Config>(hash: H256) -> Option<StoredHeader<T::AccountId>> {
	Headers::<T>::get(hash)
}

fn set_blocks_to_prune(oldest_unpruned: u64, oldest_to_keep: u64) {
	BlocksToPrune::put(PruningRange {
		oldest_unpruned_block: oldest_unpruned,
		oldest_block_to_keep: oldest_to_keep,
	});
} 

fn initialize_storage<T: Config>(
	headers: Vec<EthereumHeader>,
	initial_difficulty: U256,
) -> Result<(), &'static str> {
	let oldest_block_num = headers.get(0).ok_or("Need at least one header")?.number;
	let mut best_block_id_opt: Option<EthereumHeaderId> = None;
	let mut best_block_difficulty = initial_difficulty;

	for header in headers.iter() {
		let hash = header.compute_hash();
		let total_difficulty = {
			if oldest_block_num == header.number {
				initial_difficulty + header.difficulty
			} else {
				let parent = Headers::<T>::get(header.parent_hash).ok_or("Missing parent header")?;
				parent.total_difficulty + header.difficulty
			}
		};

		if total_difficulty > best_block_difficulty {
			best_block_difficulty = total_difficulty;
			best_block_id_opt = Some(EthereumHeaderId {
				number: header.number,
				hash: hash,
			});
		}

		Headers::<T>::insert(
			hash,
			StoredHeader {
				submitter: None,
				header: header.clone(),
				total_difficulty: total_difficulty, 
			},
		);
		HeadersByNumber::append(header.number, hash);
	}

	let best_block_id = best_block_id_opt.ok_or("Need highest difficulty block")?;
	BestBlock::put((
		best_block_id,
		best_block_difficulty,
	));

	let required_descendants = T::DescendantsUntilFinalized::get() as usize;
	let maybe_finalized_ancestor = ancestry::<T>(best_block_id.hash)
		.enumerate()
		.find_map(|(i, pair)| if i < required_descendants { None } else { Some(pair) });
	if let Some((hash, header)) = maybe_finalized_ancestor {
		FinalizedBlock::put(EthereumHeaderId {
			hash: hash,
			number: header.number,
		});
	}

	Ok(())
}

benchmarks! {
	// Benchmark `import_header` extrinsic with the worst possible conditions:
	// * Import will set a new finalized header.
	// * Import will iterate over the max value of DescendantsUntilFinalized headers
	//   in the chain.
	// * Import will prune the max number of headers, i.e. HEADERS_TO_PRUNE_IN_SINGLE_IMPORT.
	// * Pruned headers will come from distinct block numbers so that we have the max
	//   number of HeaderByNumber::take calls.
    // * The last pruned header will have siblings that we don't prune and have to
	//   re-insert using HeadersByNumber::insert.
	//
	// NOTE: Average conditions differ from worst case in that only 1 header will usually be pruned.
	import_header {
		let caller: T::AccountId = whitelisted_caller();
		let descendants_until_final = T::DescendantsUntilFinalized::get() as u64;
		// 1 extra because there's a sibling header at 8th index
		let next_finalized_idx = HEADERS_TO_PRUNE_IN_SINGLE_IMPORT as usize + 1;
		let next_tip_idx = next_finalized_idx + descendants_until_final as usize;
		let headers = data::headers_11963025_to_11963069();

		initialize_storage::<T>(headers[0..next_tip_idx].to_vec(), U256::zero())?;

		set_blocks_to_prune(
			headers[0].number,
			headers[next_finalized_idx].number,
		);

	}: _(RawOrigin::Signed(caller.clone()), headers[next_tip_idx].clone(), Default::default())
	verify {
		let best = &headers[next_tip_idx];
		assert_eq!(
			get_best_block().0,
			EthereumHeaderId {
				number: best.number,
				hash: best.compute_hash(),
			},
		);

		let finalized = &headers[next_finalized_idx];
		assert_eq!(
			get_finalized_block(),
			EthereumHeaderId {
				number: finalized.number,
				hash: finalized.compute_hash(),
			},
		);

		let last_pruned = &headers[HEADERS_TO_PRUNE_IN_SINGLE_IMPORT as usize];
		let first_pruned = &headers[0];
		assert_eq!(
			get_header::<T>(last_pruned.compute_hash()),
			None,
		);
		assert_eq!(
			get_header::<T>(first_pruned.compute_hash()),
			None,
		);
	}
}

impl_benchmark_test_suite!(
	VerifierLightclient,
	crate::mock::new_tester::<crate::mock::mock_verifier_with_pow::Test>(),
	crate::mock::mock_verifier_with_pow::Test,
);
