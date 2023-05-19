pub use crate::config::{
	SLOTS_PER_HISTORICAL_ROOT, SYNC_COMMITTEE_BITS_SIZE as SC_BITS_SIZE,
	SYNC_COMMITTEE_SIZE as SC_SIZE,
};
use frame_support::storage::types::OptionQuery;
use snowbridge_core::RingBufferMapImpl;

// Specialize types based on configured sync committee size
pub type CheckpointUpdate = primitives::CheckpointUpdate<SC_SIZE>;
pub type ExecutionHeaderUpdate = primitives::ExecutionHeaderUpdate;
pub type SyncCommitteeUpdate = primitives::SyncCommitteeUpdate<SC_SIZE, SC_BITS_SIZE>;
pub type FinalizedHeaderUpdate = primitives::FinalizedHeaderUpdate<SC_SIZE, SC_BITS_SIZE>;
pub type SyncCommittee = primitives::SyncCommittee<SC_SIZE>;
pub type SyncCommitteePrepared = primitives::SyncCommitteePrepared<SC_SIZE>;
pub type SyncAggregate = primitives::SyncAggregate<SC_SIZE, SC_BITS_SIZE>;

/// ExecutionHeader ring buffer implementation
pub(crate) type ExecutionHeaderBuffer<T> = RingBufferMapImpl<
	u32,
	<T as crate::Config>::MaxExecutionHeadersToKeep,
	crate::ExecutionHeaderIndex<T>,
	crate::ExecutionHeaderMapping<T>,
	crate::ExecutionHeaders<T>,
	OptionQuery,
>;

/// Sync committee ring buffer implementation
pub(crate) type SyncCommitteesBuffer<T> = RingBufferMapImpl<
	u32,
	<T as crate::Config>::MaxSyncCommitteesToKeep,
	crate::SyncCommitteesIndex<T>,
	crate::SyncCommitteesMapping<T>,
	crate::SyncCommittees<T>,
	OptionQuery,
>;
