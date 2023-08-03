#![no_main]
extern crate snowbridge_ethereum_beacon_client;

use std::convert::TryInto;
use snowbridge_ethereum_beacon_client::mock::minimal::*;
use snowbridge_ethereum_beacon_client::types::CheckpointUpdate;
use libfuzzer_sys::fuzz_target;
use snowbridge_ethereum_beacon_client_fuzz::types::FuzzCheckpointUpdate;

fuzz_target!(|input: FuzzCheckpointUpdate| {
   new_tester().execute_with(|| {
		let update: CheckpointUpdate = input.try_into().unwrap();
        let result = EthereumBeaconClient::process_checkpoint_update(&update);
		assert!(result.is_err());
	});
});

