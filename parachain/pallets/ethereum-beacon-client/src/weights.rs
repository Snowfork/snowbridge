// Modified to return hardcoded values for snowbase

// Executed Command:
// ./target/release/snowbridge
// benchmark
// pallet
// --chain
// /tmp/snowbridge/spec.json
// --execution=wasm
// --pallet
// ethereum_beacon_client
// --extrinsic
// *
// --steps
// 10
// --repeat
// 10
// --output
// pallets/ethereum-beacon-client/src/weights.rs
// --template
// module-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;
use crate::config;

/// Weight functions needed for ethereum_beacon_client.
pub trait WeightInfo {	fn sync_committee_period_update() -> Weight;	fn import_finalized_header() -> Weight;	fn import_execution_header() -> Weight;}

/// Weights for ethereum_beacon_client using the Snowbridge node and recommended hardware.
pub struct SnowbridgeWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SnowbridgeWeight<T> {	
	
	fn sync_committee_period_update() -> Weight {
		match config::IS_MINIMAL {
			true => (49_663_503_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(4 as Weight))			
				.saturating_add(T::DbWeight::get().writes(3 as Weight)),
			false => (170_683_416_000 as Weight)
				.saturating_add(T::DbWeight::get().reads(4 as Weight))
				.saturating_add(T::DbWeight::get().writes(2 as Weight)),
		}
	}	

	fn import_finalized_header() -> Weight {
		match config::IS_MINIMAL {
			true => (49_243_511_000 as Weight)			
				.saturating_add(T::DbWeight::get().reads(3 as Weight))			
				.saturating_add(T::DbWeight::get().writes(3 as Weight)),
			false => (166_954_505_000 as Weight)			
				.saturating_add(T::DbWeight::get().reads(3 as Weight))
				.saturating_add(T::DbWeight::get().writes(1 as Weight)),	
		}
	}	

	fn import_execution_header() -> Weight {
		match config::IS_MINIMAL {
			true => (49_538_552_000 as Weight)			
				.saturating_add(T::DbWeight::get().reads(4 as Weight))			
				.saturating_add(T::DbWeight::get().writes(2 as Weight)),
			false => (162_194_827_000 as Weight)		
				.saturating_add(T::DbWeight::get().reads(3 as Weight))		
				.saturating_add(T::DbWeight::get().writes(1 as Weight))	
		}
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {	
	fn sync_committee_period_update() -> Weight {
		match config::IS_MINIMAL {
			true => (49_663_503_000 as Weight)			
				.saturating_add(RocksDbWeight::get().reads(4 as Weight))			
				.saturating_add(RocksDbWeight::get().writes(3 as Weight)),
			false => (170_683_416_000 as Weight)		
				.saturating_add(RocksDbWeight::get().reads(4 as Weight))		
				.saturating_add(RocksDbWeight::get().writes(2 as Weight))	
		}
	}	
		
	fn import_finalized_header() -> Weight {
		match config::IS_MINIMAL {
			true => (49_243_511_000 as Weight)			
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))			
				.saturating_add(RocksDbWeight::get().writes(3 as Weight)),
			false => (166_954_505_000 as Weight)			
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))			
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))	
		}
	}	
		
	fn import_execution_header() -> Weight {
		match config::IS_MINIMAL {
			true => (49_538_552_000 as Weight)			
				.saturating_add(RocksDbWeight::get().reads(4 as Weight))			
				.saturating_add(RocksDbWeight::get().writes(2 as Weight)),
			false => (162_194_827_000 as Weight)			
				.saturating_add(RocksDbWeight::get().reads(3 as Weight))			
				.saturating_add(RocksDbWeight::get().writes(1 as Weight))	
		}
	}
}
