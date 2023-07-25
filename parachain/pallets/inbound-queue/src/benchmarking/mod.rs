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

	const GATEWAY_ADDRESS: [u8; 20] = hex!["eda338e4dc46038493b885327842fd3e301cab39"];

	#[benchmark]
	fn submit() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		let create_message = make_create_message();

		T::Helper::initialize_storage(
			create_message.message.proof.block_hash,
			create_message.execution_header,
		);

		<Gateway<T>>::put(H160(GATEWAY_ADDRESS));

		let dest_para: ParaId = 1000u32.into();
		let sovereign_account = dest_para.into_account_truncating();

		let minimum_balance = T::Token::minimum_balance();
		let minimum_balance_u32: u32 = minimum_balance.try_into()
			.unwrap_or_else(|_| panic!("unable to cast minimum balance to u32"));

		// Make sure the sovereign balance is enough
		let sovereign_balance = minimum_balance_u32 * 5;

		// So that the receiving account exists
		let _ = T::Token::mint_into(&caller, minimum_balance.into());
		// Fund the sovereign account (parachain sovereign account) so it can transfer a reward
		// fee to the caller account
		let _ = T::Token::mint_into(&sovereign_account, sovereign_balance.into());

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
}
