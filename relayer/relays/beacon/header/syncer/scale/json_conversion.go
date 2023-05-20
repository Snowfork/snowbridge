package scale

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/util"
)

func (p BeaconCheckpoint) ToJSON() json.CheckPoint {
	return json.CheckPoint{
		Header:                     p.Header.ToJSON(),
		CurrentSyncCommittee:       p.CurrentSyncCommittee.ToJSON(),
		CurrentSyncCommitteeBranch: util.ScaleBranchToString(p.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             p.ValidatorsRoot.Hex(),
		ImportTime:                 uint64(p.ImportTime),
		BlockRootsRoot:             p.BlockRootsRoot.Hex(),
		BlockRootsBranch:           util.ScaleBranchToString(p.BlockRootsBranch),
	}
}

func (p SyncCommitteePeriodPayload) ToJSON() json.SyncCommitteeUpdate {
	return json.SyncCommitteeUpdate{
		AttestedHeader:          p.AttestedHeader.ToJSON(),
		NextSyncCommittee:       p.NextSyncCommittee.ToJSON(),
		NextSyncCommitteeBranch: util.ScaleBranchToString(p.NextSyncCommitteeBranch),
		FinalizedHeader:         p.FinalizedHeader.ToJSON(),
		FinalityBranch:          util.ScaleBranchToString(p.FinalityBranch),
		SyncAggregate:           p.SyncAggregate.ToJSON(),
		SignatureSlot:           uint64(p.SignatureSlot),
		BlockRootsRoot:          p.BlockRootsRoot.Hex(),
		BlockRootsBranch:        util.ScaleBranchToString(p.BlockRootsBranch),
	}
}

func (p FinalizedHeaderPayload) ToJSON() json.FinalizedHeaderUpdate {
	return json.FinalizedHeaderUpdate{
		AttestedHeader:   p.AttestedHeader.ToJSON(),
		FinalizedHeader:  p.FinalizedHeader.ToJSON(),
		FinalityBranch:   util.ScaleBranchToString(p.FinalityBranch),
		SyncAggregate:    p.SyncAggregate.ToJSON(),
		SignatureSlot:    uint64(p.SignatureSlot),
		BlockRootsRoot:   p.BlockRootsRoot.Hex(),
		BlockRootsBranch: util.ScaleBranchToString(p.BlockRootsBranch),
	}
}

func (h HeaderUpdate) ToJSON() json.HeaderUpdate {
	var ancestryProof *json.AncestryProof
	if h.Payload.AncestryProof.HasValue {
		ancestryProof = &json.AncestryProof {
			HeaderBranch: util.ScaleBranchToString(h.Payload.AncestryProof.Value.HeaderBranch),
			FinalizedBlockRoot:   h.Payload.AncestryProof.Value.FinalizedBlockRoot.Hex(),
		}
	}
	return json.HeaderUpdate{
		Header:           h.Payload.Header.ToJSON(),
		AncestryProof:    ancestryProof,
		ExecutionHeader:  h.Payload.ExecutionHeader.ToJSON(),
		ExecutionBranch:  util.ScaleBranchToString(h.Payload.ExecutionBranch),
	}
}

func (b *BeaconHeader) ToJSON() json.BeaconHeader {
	return json.BeaconHeader{
		Slot:          uint64(b.Slot),
		ProposerIndex: uint64(b.ProposerIndex),
		ParentRoot:    b.ParentRoot.Hex(),
		StateRoot:     b.StateRoot.Hex(),
		BodyRoot:      b.BodyRoot.Hex(),
	}
}

func (e *ExecutionPayloadHeaderCapella) ToJSON() json.ExecutionPayloadHeaderCapella {
	return json.ExecutionPayloadHeaderCapella{
		ParentHash:      e.ParentHash.Hex(),
		FeeRecipient:    util.BytesToHexString(e.FeeRecipient[:]),
		StateRoot:       e.StateRoot.Hex(),
		ReceiptsRoot:    e.ReceiptsRoot.Hex(),
		LogsBloom:       util.BytesToHexString(e.LogsBloom),
		PrevRandao:      e.PrevRandao.Hex(),
		BlockNumber:     uint64(e.BlockNumber),
		GasLimit:        uint64(e.GasLimit),
		GasUsed:         uint64(e.GasUsed),
		Timestamp:       uint64(e.Timestamp),
		ExtraData:       util.BytesToHexString(e.ExtraData),
		BaseFeePerGas:   e.BaseFeePerGas.Uint64(),
		BlockHash:       e.BlockHash.Hex(),
		TransactionRoot: e.TransactionsRoot.Hex(),
		WithdrawalsRoot: e.WithdrawalsRoot.Hex(),
	}
}

func (s *SyncCommittee) ToJSON() json.SyncCommittee {
	pubkeys := []string{}
	for _, pubkeyScale := range s.Pubkeys {
		pubkeys = append(pubkeys, util.BytesToHexString(pubkeyScale[:]))
	}

	return json.SyncCommittee{
		Pubkeys:         pubkeys,
		AggregatePubkey: util.BytesToHexString(s.AggregatePubkey[:]),
	}
}

func (s *SyncAggregate) ToJSON() json.SyncAggregate {
	return json.SyncAggregate{
		SyncCommitteeBits:      util.BytesToHexString(s.SyncCommitteeBits),
		SyncCommitteeSignature: util.BytesToHexString(s.SyncCommitteeSignature[:]),
	}
}

func (a *AttestationData) ToJSON() json.AttestationData {
	return json.AttestationData{
		Slot:            uint64(a.Slot),
		Index:           uint64(a.Index),
		BeaconBlockRoot: a.BeaconBlockRoot.Hex(),
		Source: json.Checkpoint{
			Epoch: uint64(a.Source.Epoch),
			Root:  a.Source.Root.Hex(),
		},
		Target: json.Checkpoint{
			Epoch: uint64(a.Target.Epoch),
			Root:  a.Target.Root.Hex(),
		},
	}
}
