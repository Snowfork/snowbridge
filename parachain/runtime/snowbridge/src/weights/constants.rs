use frame_support::{
    weights::{RuntimeDbWeight, Weight},
    weights::constants::{WEIGHT_PER_MILLIS, WEIGHT_PER_MICROS},
}

/// Weight of importing a block with 0 txs
pub const BLOCK_EXECUTION_WEIGHT: Weight = {{ block_execution_weight_in_millis }} * WEIGHT_PER_MILLIS;
/// Weight of executing 10,000 System remarks (no-op) txs
pub const EXTRINSIC_BASE_WEIGHT: Weight = {{ extrinsic_base_weight_in_micros }} * WEIGHT_PER_MICROS;
/// Weight of reads and writes to RocksDB, the default DB used by Sgitubstrate
pub const ROCKS_DB_WEIGHT: RuntimeDbWeight = RuntimeDbWeight {
    read: {{ rocksdb_read_weight_in_micros }} * WEIGHT_PER_MICROS,
    write: {{ rocksdb_write_weight_in_micros }} * WEIGHT_PER_MICROS,
};
