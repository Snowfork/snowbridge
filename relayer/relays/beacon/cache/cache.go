package cache

import (
	ssz "github.com/ferranbt/fastssz"
	"sync"

	"github.com/ethereum/go-ethereum/common"
)

type Finalized struct {
	// Stores the last successfully imported hash
	LastSyncedHash common.Hash
	// Stores the last successfully synced slot
	LastSyncedSlot uint64
	// Stores the last attempted finalized header, whether the import succeeded or not.
	LastAttemptedSyncHash common.Hash
	// Stores the slot number of the above header
	LastAttemptedSyncSlot uint64
	// BlockRoots
	BlockRootsTrees map[common.Hash]*ssz.Node
}

type Checkpoint struct {
	BlockRoot      common.Hash
	BlockRootsTree *ssz.Node
	Slot           uint64
	Period         uint64
}

type FinalizedCheckpoints struct {
	Slots       []uint64
	CheckPoints map[uint64]Checkpoint
}

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	Finalized                     Finalized
	FinalizedCheckPoints          FinalizedCheckpoints
	mu                            sync.Mutex
}

func New() *BeaconCache {
	return &BeaconCache{
		Finalized: Finalized{
			BlockRootsTrees: make(map[common.Hash]*ssz.Node),
		},
		FinalizedCheckPoints: FinalizedCheckpoints{
			CheckPoints: make(map[uint64]Checkpoint),
		},
	}
}

func (b *BeaconCache) SetLastSyncedSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutex lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	if period > b.LastSyncedSyncCommitteePeriod {
		b.LastSyncedSyncCommitteePeriod = period
	}
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.Finalized.LastSyncedHash
}
