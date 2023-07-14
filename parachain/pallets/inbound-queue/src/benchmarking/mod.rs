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
	use crate::benchmarking::fixtures::make_create_message;
	use hex_literal::hex;

	const OUTBOUND_QUEUE_ADDRESS: [u8; 20] = hex!["ee9170abfbf9421ad6dd07f6bdec9d89f2b581e0"];

	#[benchmark]
	fn submit() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let create_message = make_create_message();

		T::Helper::initialize_storage(
			create_message.message.proof.block_hash,
			create_message.execution_header,
		);

		<AllowList<T>>::put(create_allowlist::<T>());

		let dest_para: ParaId = 1000u32.into();
		let sovereign_account = dest_para.into_account_truncating();

		// So that the receiving account exists
		let _ = T::Token::mint_into(&caller, T::Token::minimum_balance().into());
		// Fund the sovereign account (parachain sovereign account) so it can transfer a reward
		// fee to the caller account
		let _ = T::Token::mint_into(&sovereign_account, 10000u32.into());

		#[block]
		{
			let _ = InboundQueue::<T>::submit(
				RawOrigin::Signed(caller.clone()).into(),
				create_message.message,
			)?;
		}

		Ok(())
	}

	impl_benchmark_test_suite!(
		InboundQueue,
		crate::test::new_tester(crate::H160::default()),
		crate::test::Test
	);

	fn create_allowlist<T>() -> BoundedBTreeSet<H160, T::AllowListLength>
	where
		T: Config,
	{
		let allowlist: BoundedBTreeSet<H160, T::AllowListLength> =
			BTreeSet::from_iter(vec![OUTBOUND_QUEUE_ADDRESS.into()].into_iter())
				.try_into()
				.expect("exceeded bound");

		allowlist
	}
}
