#![no_main]
extern crate snowbridge_ethereum_beacon_client;

use snowbridge_beacon_primitives::updates::AncestryProof;
use snowbridge_beacon_primitives::ExecutionHeaderUpdate;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: FuzzExecutionHeaderUpdate| {
   new_tester().execute_with(|| {
		let update: ExecutionHeaderUpdate = input.try_into().unwrap();
        let result = EthereumBeaconClient::process_execution_header_update(&update);
		assert!(result.is_err());
	});
});

