package syncer

import "github.com/snowfork/snowbridge/relayer/relays/beacon/config"

func (s *Syncer) ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (s.SlotsInEpoch * s.EpochsPerSyncCommitteePeriod)
}

func (s *Syncer) ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / s.SlotsInEpoch
}

func (s *Syncer) IsStartOfEpoch(slot uint64) bool {
	return slot%s.SlotsInEpoch == 0
}

func (s *Syncer) CalculateNextCheckpointSlot(slot uint64) uint64 {
	syncPeriod := s.ComputeSyncPeriodAtSlot(slot)

	// on new period boundary
	if syncPeriod*s.SlotsInEpoch*s.EpochsPerSyncCommitteePeriod == slot {
		return slot
	}

	return (syncPeriod + 1) * s.SlotsInEpoch * s.EpochsPerSyncCommitteePeriod
}

func (s *Syncer) IsCapellaForking(slot uint64) bool {
	epoch := s.ComputeEpochAtSlot(slot)
	if s.activeSpec == config.Minimal && epoch >= config.Minimal_CapellaForkEpoch {
		return true
	}
	if s.activeSpec == config.GOERLI && epoch >= config.Goerli_CapellaForkEpoch {
		return true
	}
	if s.activeSpec == config.Mainnet && epoch >= config.Mainnet_CapellaForkEpoch {
		return true
	}
	return false
}
