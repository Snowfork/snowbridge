package protocol

import "github.com/snowfork/snowbridge/relayer/relays/beacon/config"

type Protocol struct {
	Settings config.SpecSettings
}

func New(setting config.SpecSettings) *Protocol {
	return &Protocol{Settings: setting}
}

func (p *Protocol) ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (p.Settings.SlotsInEpoch * p.Settings.EpochsPerSyncCommitteePeriod)
}

func (p *Protocol) ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / p.Settings.SlotsInEpoch
}

func (p *Protocol) IsStartOfEpoch(slot uint64) bool {
	return slot%p.Settings.SlotsInEpoch == 0
}

func (p *Protocol) CalculateNextCheckpointSlot(slot uint64) uint64 {
	syncPeriod := p.ComputeSyncPeriodAtSlot(slot)

	// on new period boundary
	if syncPeriod*p.Settings.SlotsInEpoch*p.Settings.EpochsPerSyncCommitteePeriod == slot {
		return slot
	}

	return (syncPeriod + 1) * p.Settings.SlotsInEpoch * p.Settings.EpochsPerSyncCommitteePeriod
}

func (p *Protocol) DenebForked(slot uint64) bool {
	return p.ComputeEpochAtSlot(slot) >= p.Settings.DenebForkEpoch
}

func (p *Protocol) SyncPeriodLength() uint64 {
	return p.Settings.SlotsInEpoch * p.Settings.EpochsPerSyncCommitteePeriod
}
