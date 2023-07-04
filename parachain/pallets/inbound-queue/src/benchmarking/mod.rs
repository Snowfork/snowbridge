mod fixtures;

// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
use super::*;

use crate::Pallet as InboundQueue;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;
	use crate::benchmarking::fixtures::{make_create_message, make_mint_message};
	use hex_literal::hex;

	const OUTBOUND_QUEUE_ADDRESS: [u8; 20] = hex!["ee9170abfbf9421ad6dd07f6bdec9d89f2b581e0"];

	#[benchmark]
	fn submit_create_message() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let create_message = make_create_message();

		// TODO I've tried removing this and setting ExecutionHeaderBuffer directly in
		// storage as Vincent suggested, but that doesn't seem to work, can't import the
		// beacon pallet here directly. I've tried setting it in new_tester_with_config(),
		// branch clara/sno-515-set-storage-directly
		// but same issue as the allowlist, doesn't seem to actually populating storage.
		T::Verifier::initialize_storage(
			create_message.message.proof.block_hash,
			create_message.execution_header,
		);

		// TODO: The allowlist set in new_tester_with_config() doesn't seem to have any effect.
		// If I leave the allowlist setting in these lines out, I get a InvalidOutboundQueue error.
		let allowlist: BoundedBTreeSet<H160, T::AllowListLength> =
			BTreeSet::from_iter(vec![OUTBOUND_QUEUE_ADDRESS.into()].into_iter())
				.try_into()
				.expect("exceeded bound");
		<AllowList<T>>::put(allowlist);

		let _ = T::Token::mint_into(&caller, T::Token::minimum_balance().into());

		#[block]
		{
			// TODO: Calling this results in xcm failed: Transport("NoChannel"). Not sure
			// if that is a problem.
			let _ = InboundQueue::<T>::submit(
				RawOrigin::Signed(caller.clone()).into(),
				create_message.message,
			)?;
		}

		Ok(())
	}

	#[benchmark]
	fn submit_mint_message() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let mint_message = make_mint_message();

		T::Verifier::initialize_storage(
			mint_message.message.proof.block_hash,
			mint_message.execution_header,
		);

		<Nonce<T>>::set(ParaId::new(1000u32), 1);

		let allowlist: BoundedBTreeSet<H160, T::AllowListLength> =
			BTreeSet::from_iter(vec![OUTBOUND_QUEUE_ADDRESS.into()].into_iter())
				.try_into()
				.expect("exceeded bound");
		<AllowList<T>>::put(allowlist);

		let _ = T::Token::mint_into(&caller, T::Token::minimum_balance().into());

		#[block]
		{
			let _ = InboundQueue::<T>::submit(
				RawOrigin::Signed(caller.clone()).into(),
				mint_message.message,
			)?;
		}

		Ok(())
	}

	impl_benchmark_test_suite!(
		InboundQueue,
		crate::test::new_tester(OUTBOUND_QUEUE_ADDRESS.into()),
		crate::test::Test
	);
}
