package protocol

// EIP-7688 turns BeaconState and BeaconBlockBody into progressive containers, so every index
// below differs from Electra even where the field itself is unchanged. The execution index is
// also a different field: EIP-7732 removes execution_payload from the body, leaving the bid's
// parent_block_hash.
const (
	GloasBlockRootGeneralizedIndex            = 352
	GloasFinalizedCheckpointGeneralizedIndex  = 735
	GloasCurrentSyncCommitteeGeneralizedIndex = 2945
	GloasNextSyncCommitteeGeneralizedIndex    = 2946
	GloasExecutionPayloadGeneralizedIndex     = 2856
)
