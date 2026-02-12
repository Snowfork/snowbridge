package mock

import (
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/scale"
)

// StateService is a mock implementation for testing
type StateService struct {
	BlockRootProofs       map[uint64]*scale.BlockRootProof
	FinalizedHeaderProofs map[uint64][]types.H256
	SyncCommitteeProofs   map[string]*scale.SyncCommitteeProof
	HealthError           error
}

func (m *StateService) GetBlockRootProof(slot uint64) (*scale.BlockRootProof, error) {
	if proof, ok := m.BlockRootProofs[slot]; ok {
		return proof, nil
	}
	return &scale.BlockRootProof{}, nil
}

func (m *StateService) GetFinalizedHeaderProof(slot uint64) ([]types.H256, error) {
	if proof, ok := m.FinalizedHeaderProofs[slot]; ok {
		return proof, nil
	}
	return []types.H256{}, nil
}

func (m *StateService) GetSyncCommitteeProof(slot uint64, period string) (*scale.SyncCommitteeProof, error) {
	if m.SyncCommitteeProofs == nil {
		return &scale.SyncCommitteeProof{}, nil
	}
	key := period
	if proof, ok := m.SyncCommitteeProofs[key]; ok {
		return proof, nil
	}
	return &scale.SyncCommitteeProof{}, nil
}

func (m *StateService) Health() error {
	return m.HealthError
}
