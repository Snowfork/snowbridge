#![no_main]
extern crate snowbridge_ethereum_client_pallet;

use snowbridge_beacon_primitives::ExecutionHeaderUpdate;
use snowbridge_ethereum_client_pallet::mock::minimal::*;
use snowbridge_ethereum_client_pallet::types::FuzzExecutionHeaderUpdate;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: FuzzExecutionHeaderUpdate| {
	new_tester().execute_with(|| {
		let update: ExecutionHeaderUpdate = input.try_into().unwrap();
		let result = EthereumBeaconClient::submit_execution_header(
			RuntimeOrigin::signed(1),
			Box::new(update),
		);
		assert!(result.is_err());
	});
});