package scale

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/json"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/header/syncer/util"
)

func (s InitialSync) ToJSON() json.InitialSync {
	return json.InitialSync{
		Header:                     s.Header.ToJSON(),
		CurrentSyncCommittee:       s.CurrentSyncCommittee.ToJSON(),
		CurrentSyncCommitteeBranch: util.ScaleBranchToString(s.CurrentSyncCommitteeBranch),
		ValidatorsRoot:             s.ValidatorsRoot.Hex(),
		ImportTime:                 uint64(s.ImportTime),
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
		SyncCommitteePeriod:     uint64(p.SyncCommitteePeriod),
		SignatureSlot:           uint64(p.SignatureSlot),
		BlockRootsHash:          p.BlockRootsHash.Hex(),
		BlockRootProof:          util.ScaleBranchToString(p.BlockRootProof),
	}
}

func (p FinalizedHeaderPayload) ToJSON() json.FinalizedHeaderUpdate {
	return json.FinalizedHeaderUpdate{
		AttestedHeader:  p.AttestedHeader.ToJSON(),
		FinalizedHeader: p.FinalizedHeader.ToJSON(),
		FinalityBranch:  util.ScaleBranchToString(p.FinalityBranch),
		SyncAggregate:   p.SyncAggregate.ToJSON(),
		SignatureSlot:   uint64(p.SignatureSlot),
		BlockRootsHash:  p.BlockRootsHash.Hex(),
		BlockRootProof:  util.ScaleBranchToString(p.BlockRootProof),
	}
}

func (h HeaderUpdate) ToJSON() json.HeaderUpdate {
	proposerSlashings := []json.ProposerSlashing{}
	for _, proposerSlashing := range h.Block.Body.ProposerSlashings {
		proposerSlashings = append(proposerSlashings, json.ProposerSlashing{
			SignedHeader1: json.SignedHeader{
				Message:   proposerSlashing.SignedHeader1.Message.ToJSON(),
				Signature: util.BytesToHexString(proposerSlashing.SignedHeader1.Signature),
			},
			SignedHeader2: json.SignedHeader{
				Message:   proposerSlashing.SignedHeader2.Message.ToJSON(),
				Signature: util.BytesToHexString(proposerSlashing.SignedHeader2.Signature),
			},
		})
	}

	attesterSlashings := []json.AttesterSlashing{}
	for _, attesterSlashing := range h.Block.Body.AttesterSlashings {
		attesterSlashings = append(attesterSlashings, json.AttesterSlashing{
			Attestation1: json.IndexedAttestation{
				AttestingIndices: util.ToUint64Array(attesterSlashing.Attestation1.AttestingIndices),
				Data:             attesterSlashing.Attestation1.Data.ToJSON(),
				Signature:        util.BytesToHexString(attesterSlashing.Attestation1.Signature),
			},
			Attestation2: json.IndexedAttestation{
				AttestingIndices: util.ToUint64Array(attesterSlashing.Attestation2.AttestingIndices),
				Data:             attesterSlashing.Attestation2.Data.ToJSON(),
				Signature:        util.BytesToHexString(attesterSlashing.Attestation2.Signature),
			},
		})
	}

	attestations := []json.Attestation{}
	for _, attestation := range h.Block.Body.Attestations {
		attestations = append(attestations, json.Attestation{
			AggregationBits: util.BytesToHexString(attestation.AggregationBits),
			Data:            attestation.Data.ToJSON(),
			Signature:       util.BytesToHexString(attestation.Signature),
		})
	}

	deposits := []json.Deposit{}
	for _, deposit := range h.Block.Body.Deposits {
		deposits = append(deposits, json.Deposit{
			Proof: util.ScaleBranchToString(deposit.Proof),
			Data: json.DepositData{
				Pubkey:                util.BytesToHexString(deposit.Data.Pubkey),
				WithdrawalCredentials: deposit.Data.WithdrawalCredentials.Hex(),
				Amount:                uint64(deposit.Data.Amount),
				Signature:             util.BytesToHexString(deposit.Data.Signature),
			},
		})
	}

	voluntaryExits := []json.VoluntaryExit{}
	for _, voluntaryExit := range h.Block.Body.VoluntaryExits {
		voluntaryExits = append(voluntaryExits, json.VoluntaryExit{
			Epoch:          uint64(voluntaryExit.Epoch),
			ValidatorIndex: uint64(voluntaryExit.ValidaterIndex),
		})
	}

	return json.HeaderUpdate{
		Block: json.Block{
			Slot:          uint64(h.Block.Slot),
			ProposerIndex: uint64(h.Block.ProposerIndex),
			ParentRoot:    h.Block.ParentRoot.Hex(),
			StateRoot:     h.Block.StateRoot.Hex(),
			Body: json.BlockBody{
				RandaoReveal: util.BytesToHexString(h.Block.Body.RandaoReveal),
				Eth1Data: json.Eth1Data{
					DepositRoot:  h.Block.Body.Eth1Data.DepositRoot.Hex(),
					DepositCount: uint64(h.Block.Body.Eth1Data.DepositCount),
					BlockHash:    h.Block.Body.Eth1Data.BlockHash.Hex(),
				},
				Graffiti:          h.Block.Body.Graffiti.Hex(),
				ProposerSlashings: proposerSlashings,
				AttesterSlashings: attesterSlashings,
				Attestations:      attestations,
				Deposits:          deposits,
				VoluntaryExits:    voluntaryExits,
				SyncAggregate:     h.Block.Body.SyncAggregate.ToJSON(),
				ExecutionPayload: json.ExecutionPayload{
					ParentHash:      h.Block.Body.ExecutionPayload.ParentHash.Hex(),
					FeeRecipient:    util.BytesToHexString(h.Block.Body.ExecutionPayload.FeeRecipient),
					StateRoot:       h.Block.Body.ExecutionPayload.StateRoot.Hex(),
					ReceiptsRoot:    h.Block.Body.ExecutionPayload.ReceiptsRoot.Hex(),
					LogsBloom:       util.BytesToHexString(h.Block.Body.ExecutionPayload.LogsBloom),
					PrevRandao:      h.Block.Body.ExecutionPayload.PrevRandao.Hex(),
					BlockNumber:     uint64(h.Block.Body.ExecutionPayload.BlockNumber),
					GasLimit:        uint64(h.Block.Body.ExecutionPayload.GasLimit),
					GasUsed:         uint64(h.Block.Body.ExecutionPayload.GasUsed),
					Timestamp:       uint64(h.Block.Body.ExecutionPayload.Timestamp),
					ExtraData:       util.BytesToHexString(h.Block.Body.ExecutionPayload.ExtraData),
					BaseFeePerGas:   h.Block.Body.ExecutionPayload.BaseFeePerGas.Uint64(),
					BlockHash:       h.Block.Body.ExecutionPayload.BlockHash.Hex(),
					TransactionRoot: h.Block.Body.ExecutionPayload.TransactionsRoot.Hex(),
				},
			},
		},
		SyncAggregate:                 h.SyncAggregate.ToJSON(),
		SignatureSlot:                 uint64(h.SignatureSlot),
		BlockRootProof:                util.ScaleBranchToString(h.BlockRootProof),
		BlockRootProofFinalizedHeader: h.BlockRootProofFinalizedHeader.Hex(),
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
		SyncCommitteeSignature: util.BytesToHexString(s.SyncCommitteeSignature),
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
