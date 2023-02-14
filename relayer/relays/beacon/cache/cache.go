package cache

import (
	"errors"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
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
	// Stores
	Checkpoints CheckPoints
}

type State struct {
	FinalizedBlockRoot common.Hash
	BlockRootsTree     *ssz.Node
	Slot               uint64
	Period             uint64
}

type CheckPoints struct {
	Slots  []uint64
	States map[uint64]State
}

type BeaconCache struct {
	LastSyncedSyncCommitteePeriod uint64
	Finalized                     Finalized
	slotsInEpoch                  uint64
	epochsPerSyncCommitteePeriod  uint64
	mu                            sync.Mutex
}

func New(slotsInEpoch, epochsPerSyncCommitteePeriod uint64) *BeaconCache {
	return &BeaconCache{
		slotsInEpoch:                 slotsInEpoch,
		epochsPerSyncCommitteePeriod: epochsPerSyncCommitteePeriod,
		Finalized: Finalized{Checkpoints: CheckPoints{
			Slots:  []uint64{},
			States: make(map[uint64]State),
		}},
	}
}

func (b *BeaconCache) SetLastSyncedSyncCommitteePeriod(period uint64) {
	b.mu.Lock() // mutex lock since both the finalized and latest headers write to this slice in separate Goroutines
	defer b.mu.Unlock()
	if period > b.LastSyncedSyncCommitteePeriod {
		b.LastSyncedSyncCommitteePeriod = period
	}
}

func (b *BeaconCache) AddCheckPoint(finalizedHeaderRoot common.Hash, blockRootsTree *ssz.Node, slot uint64) {
	b.Finalized.Checkpoints.Slots = append(b.Finalized.Checkpoints.Slots, slot)
	b.Finalized.Checkpoints.States[slot] = State{
		FinalizedBlockRoot: finalizedHeaderRoot,
		BlockRootsTree:     blockRootsTree,
		Slot:               slot,
		Period:             slot / (b.slotsInEpoch * b.epochsPerSyncCommitteePeriod),
	}

	log.WithFields(log.Fields{
		"state": b.Finalized.Checkpoints.States[slot],
		"slot":  slot,
	}).Info("added finalized checkpoint to state cache") // TODO prune old states
}

func (b *BeaconCache) calculateClosestCheckpointSlot(slot uint64) (uint64, error) {
	blockRootThreshold := int(b.slotsInEpoch * b.epochsPerSyncCommitteePeriod)
	for _, i := range b.Finalized.Checkpoints.Slots {
		if i < slot {
			continue
		}

		if i == slot { // if the slot is at a finalized checkpoint, we don't need to do the ancestry proof
			return i, nil
		}

		checkpointSlot := int(i) // convert to int since it can be negative
		if checkpointSlot-blockRootThreshold < int(slot) {
			return i, nil
		}
	}

	return 0, errors.New("no checkpoint including slot in block roots threshold found")
}

func (b *BeaconCache) GetClosestCheckpoint(slot uint64) (State, error) {
	checkpointSlot, err := b.calculateClosestCheckpointSlot(slot)
	if err != nil {
		return State{}, err
	}

	return b.Finalized.Checkpoints.States[checkpointSlot], nil
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.Finalized.LastSyncedHash
}
