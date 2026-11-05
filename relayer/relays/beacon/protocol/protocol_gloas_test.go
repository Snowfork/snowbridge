package protocol

import (
	"testing"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/config"
)

// Each selector must return the index for the era the slot falls in, and adding Gloas must
// not disturb the earlier arms. Fulu deliberately shares Electra's indices; there is no fulu
// arm, and that is intentional rather than an omission.
func TestGeneralizedIndicesCoverEveryForkEra(t *testing.T) {
	p := New(config.SpecSettings{
		SlotsInEpoch: 32,
		ForkVersions: config.ForkVersions{Deneb: 0, Electra: 100, Fulu: 150, Gloas: 200},
	}, 0)

	for _, tc := range []struct {
		era                                           string
		epoch                                         uint64
		blockRoot, finalized, currentSC, nextSC, exec int
	}{
		{"deneb/altair", 0, 37, 105, 54, 55, AltairExecutionPayloadGeneralizedIndex},
		{"electra", 100, 69, 169, 86, 87, 25},
		{"fulu shares electra", 150, 69, 169, 86, 87, 25},
		{"gloas", 200, 352, 735, 2945, 2946, 2856},
	} {
		t.Run(tc.era, func(t *testing.T) {
			slot := tc.epoch * 32
			if got := p.BlockRootGeneralizedIndex(slot); got != tc.blockRoot {
				t.Errorf("block_roots: got %d want %d", got, tc.blockRoot)
			}
			if got := p.FinalizedCheckpointGeneralizedIndex(slot); got != tc.finalized {
				t.Errorf("finalized_checkpoint: got %d want %d", got, tc.finalized)
			}
			if got := p.CurrentSyncCommitteeGeneralizedIndex(slot); got != tc.currentSC {
				t.Errorf("current_sync_committee: got %d want %d", got, tc.currentSC)
			}
			if got := p.NextSyncCommitteeGeneralizedIndex(slot); got != tc.nextSC {
				t.Errorf("next_sync_committee: got %d want %d", got, tc.nextSC)
			}
			if got := p.ExecutionPayloadGeneralizedIndex(slot); got != tc.exec {
				t.Errorf("execution commitment: got %d want %d", got, tc.exec)
			}
		})
	}
}

// A config written before Gloas existed has no gloas key, which decodes as 0. Without a guard
// that would make every slot Gloas and hand every proof the wrong generalized index.
func TestUnsetGloasEpochDoesNotCaptureEverySlot(t *testing.T) {
	p := New(config.SpecSettings{
		SlotsInEpoch: 32,
		ForkVersions: config.ForkVersions{Deneb: 0, Electra: 100, Fulu: 150},
	}, 0)

	for _, slot := range []uint64{0, 100 * 32, 150 * 32, 1 << 40} {
		if fv := p.ForkVersion(slot); fv == Gloas {
			t.Fatalf("slot %d resolved to Gloas with no gloas epoch configured", slot)
		}
	}
	if got := p.FinalizedCheckpointGeneralizedIndex(1 << 40); got != ElectraFinalizedCheckpointGeneralizedIndex {
		t.Fatalf("expected the Electra index with gloas unset, got %d", got)
	}
}
