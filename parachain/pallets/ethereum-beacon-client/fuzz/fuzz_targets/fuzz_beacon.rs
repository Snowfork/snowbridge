#![no_main]
extern crate snowbridge_ethereum_beacon_client;
use snowbridge_ethereum_beacon_client::fuzzing::minimal::*;
use snowbridge_beacon_primitives::types::BeaconHeader;

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary;

#[derive(arbitrary::Arbitrary, Debug, Clone)]
pub struct FuzzBeaconHeader {
	pub slot: u64,
	pub proposer_index: u64,
	pub parent_root: [u8; 32],
	pub state_root: [u8; 32],
	pub body_root: [u8; 32],
	pub domain: [u8; 32]
}

impl TryFrom<FuzzBeaconHeader> for BeaconHeader
{
	type Error = String;

	fn try_from(other: FuzzBeaconHeader) -> Result<Self, Self::Error> {
		Ok(Self {
			slot: other.slot,
			proposer_index: other.proposer_index,
			parent_root: other.parent_root.into(),
			state_root: other.state_root.into(),
			body_root: other.body_root.into(),
		})
	}
}

fuzz_target!(|input: FuzzBeaconHeader| {
   new_tester().execute_with(|| {
		let beacon_header: BeaconHeader = input.clone().try_into().unwrap();
        _ = EthereumBeaconClient::compute_signing_root(&beacon_header, input.domain.into());
	});
});


