use super::*;

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use hex_literal::hex;

#[allow(unused_imports)]
use crate::Pallet as EthereumBeaconClient;

mod data;

benchmarks! {
	sync_committee_period_update {
		let caller: T::AccountId = whitelisted_caller();

		let other_sync_committee_period_update = data::sync_committee_update();
        
    }: sync_committee_period_update(RawOrigin::Signed(caller.clone()), other_sync_committee_period_update)
    verify {
        
    }
}