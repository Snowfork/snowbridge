package cache

import (
	"errors"
	"sort"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	ssz "github.com/ferranbt/fastssz"
	log "github.com/sirupsen/logrus"
)

const FinalizedCheckpointsLimit = 50

var (
	FinalizedCheckPointNotAvailable = errors.New("finalized checkpoint for block roots proof not available in cache")
	FinalizedCheckPointNotPopulated = errors.New("finalized checkpoint for slot not populated in cache yet")
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
	// Stores finalized checkpoints
	Checkpoints CheckPoints
}

type Proof struct {
	FinalizedBlockRoot common.Hash
	BlockRootsTree     *ssz.Node
	Slot               uint64
}

type CheckPoints struct {
	Slots  []uint64
	Proofs map[uint64]Proof
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
			Proofs: make(map[uint64]Proof),
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
	b.addSlot(slot)
	b.Finalized.Checkpoints.Proofs[slot] = Proof{
		FinalizedBlockRoot: finalizedHeaderRoot,
		BlockRootsTree:     blockRootsTree,
		Slot:               slot,
	}

	b.pruneOldCheckpoints()
}

func (b *BeaconCache) AddCheckPointSlots(slots []uint64) {
	for _, slot := range slots {
		b.addSlot(slot)
	}
}

func (b *BeaconCache) GetClosestCheckpoint(slot uint64) (Proof, error) {
	checkpointSlot, err := b.calculateClosestCheckpointSlot(slot)
	if err != nil {
		return Proof{}, err
	}

	val, ok := b.Finalized.Checkpoints.Proofs[checkpointSlot]
	if !ok {
		return Proof{Slot: checkpointSlot}, FinalizedCheckPointNotPopulated
	}

	return val, nil
}

func (b *BeaconCache) LastFinalizedHeader() common.Hash {
	return b.Finalized.LastSyncedHash
}

func (b *BeaconCache) pruneOldCheckpoints() {
	checkpointsCount := len(b.Finalized.Checkpoints.Slots)
	if checkpointsCount <= FinalizedCheckpointsLimit {
		return
	}

	slotsToKeep := b.Finalized.Checkpoints.Slots[checkpointsCount-FinalizedCheckpointsLimit : checkpointsCount]
	slotsToPrune := b.Finalized.Checkpoints.Slots[0 : checkpointsCount-FinalizedCheckpointsLimit]

	for _, slot := range slotsToPrune {
		delete(b.Finalized.Checkpoints.Proofs, slot)
	}

	log.WithField("prunedSlots", slotsToPrune).Info("pruned finalized checkpoint slots from cache")

	b.Finalized.Checkpoints.Slots = slotsToKeep
}

func (b *BeaconCache) addSlot(slot uint64) {
	addedAlready := false
	for _, i := range b.Finalized.Checkpoints.Slots {
		if i == slot {
			addedAlready = true
			break
		}
	}
	if !addedAlready {
		b.Finalized.Checkpoints.Slots = append(b.Finalized.Checkpoints.Slots, slot)
	}
	sort.Slice(b.Finalized.Checkpoints.Slots, func(i, j int) bool { return b.Finalized.Checkpoints.Slots[i] < b.Finalized.Checkpoints.Slots[j] })
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

	return 0, FinalizedCheckPointNotAvailable
}
