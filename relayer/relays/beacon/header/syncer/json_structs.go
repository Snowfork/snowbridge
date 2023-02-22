package syncer

import (
	"fmt"
	"strings"
)

type InitialSync struct {
	Header                     BeaconHeaderJSON  `json:"header"`
	CurrentSyncCommittee       SyncCommitteeJSON `json:"current_sync_committee"`
	CurrentSyncCommitteeBranch []string          `json:"current_sync_committee_branch"`
	ValidatorsRoot             string            `json:"validators_root"`
	ImportTime                 uint64            `json:"import_time"`
}

type BeaconHeaderJSON struct {
	Slot          uint64 `json:"slot"`
	ProposerIndex uint64 `json:"proposer_index"`
	ParentRoot    string `json:"parent_root"`
	StateRoot     string `json:"state_root"`
	BodyRoot      string `json:"body_root"`
}

type SyncCommitteeJSON struct {
	Pubkeys         []string `json:"pubkeys"`
	AggregatePubkey string   `json:"aggregate_pubkey"`
}

type SyncAggregateJSON struct {
	SyncCommitteeBits      string `json:"sync_committee_bits"`
	SyncCommitteeSignature string `json:"sync_committee_signature"`
}

type SyncCommitteePeriodPayloadJSON struct {
	AttestedHeader          BeaconHeaderJSON  `json:"attested_header"`
	NextSyncCommittee       SyncCommitteeJSON `json:"next_sync_committee"`
	NextSyncCommitteeBranch []string          `json:"next_sync_committee_branch"`
	FinalizedHeader         BeaconHeaderJSON  `json:"finalized_header"`
	FinalityBranch          []string          `json:"finality_branch"`
	SyncAggregate           SyncAggregateJSON `json:"sync_aggregate"`
	SyncCommitteePeriod     uint64            `json:"sync_committee_period"`
	SignatureSlot           uint64            `json:"signature_slot"`
	BlockRootsHash          string            `json:"block_roots_hash"`
	BlockRootProof          []string          `json:"block_roots_proof"`
}

type FinalizedHeaderPayloadJSON struct {
	AttestedHeader  BeaconHeaderJSON  `json:"attested_header"`
	FinalizedHeader BeaconHeaderJSON  `json:"finalized_header"`
	FinalityBranch  []string          `json:"finality_branch"`
	SyncAggregate   SyncAggregateJSON `json:"sync_aggregate"`
	SignatureSlot   uint64            `json:"signature_slot"`
	BlockRootsHash  string            `json:"block_roots_hash"`
	BlockRootProof  []string          `json:"block_roots_proof"`
}

type ProposerSlashingJSON struct {
	SignedHeader1 SignedHeaderJSON `json:"signed_header_1"`
	SignedHeader2 SignedHeaderJSON `json:"signed_header_2"`
}

type AttesterSlashingJSON struct {
	Attestation1 IndexedAttestationJSON `json:"attestation_1"`
	Attestation2 IndexedAttestationJSON `json:"attestation_2"`
}

type IndexedAttestationJSON struct {
	AttestingIndices []uint64            `json:"attesting_indices"`
	Data             AttestationDataJSON `json:"data"`
	Signature        string              `json:"signature"`
}

type AttestationDataJSON struct {
	Slot            uint64         `json:"slot"`
	Index           uint64         `json:"index"`
	BeaconBlockRoot string         `json:"beacon_block_root"`
	Source          CheckpointJSON `json:"source"`
	Target          CheckpointJSON `json:"target"`
}

type CheckpointJSON struct {
	Epoch uint64 `json:"epoch"`
	Root  string `json:"root"`
}

type SignedHeaderJSON struct {
	Message   BeaconHeaderJSON `json:"message"`
	Signature string           `json:"signature"`
}

type BlockJSON struct {
	Slot          uint64        `json:"slot"`
	ProposerIndex uint64        `json:"proposer_index"`
	ParentRoot    string        `json:"parent_root"`
	StateRoot     string        `json:"state_root"`
	Body          BlockBodyJSON `json:"body"`
}

type ExecutionPayloadJSON struct {
	ParentHash      string `json:"parent_hash"`
	FeeRecipient    string `json:"fee_recipient"`
	StateRoot       string `json:"state_root"`
	ReceiptsRoot    string `json:"receipts_root"`
	LogsBloom       string `json:"logs_bloom"`
	PrevRandao      string `json:"prev_randao"`
	BlockNumber     uint64 `json:"block_number"`
	GasLimit        uint64 `json:"gas_limit"`
	GasUsed         uint64 `json:"gas_used"`
	Timestamp       uint64 `json:"timestamp"`
	ExtraData       string `json:"extra_data"`
	BaseFeePerGas   uint64 `json:"base_fee_per_gas"`
	BlockHash       string `json:"block_hash"`
	TransactionRoot string `json:"transactions_root"`
}

type Eth1DataJSON struct {
	DepositRoot  string `json:"deposit_root"`
	DepositCount uint64 `json:"deposit_count"`
	BlockHash    string `json:"block_hash"`
}

type BlockBodyJSON struct {
	RandaoReveal      string                 `json:"randao_reveal"`
	Eth1Data          Eth1DataJSON           `json:"eth1_data"`
	Graffiti          string                 `json:"graffiti"`
	ProposerSlashings []ProposerSlashingJSON `json:"proposer_slashings"`
	AttesterSlashings []AttesterSlashingJSON `json:"attester_slashings"`
	Attestations      []AttestationJSON      `json:"attestations"`
	Deposits          []DepositJSON          `json:"deposits"`
	VoluntaryExits    []VoluntaryExitJSON    `json:"voluntary_exits"`
	SyncAggregate     SyncAggregateJSON      `json:"sync_aggregate"`
	ExecutionPayload  ExecutionPayloadJSON   `json:"execution_payload"`
}

type HeaderUpdatePayloadJSON struct {
	Block                         BlockJSON         `json:"block"`
	SyncAggregate                 SyncAggregateJSON `json:"sync_aggregate"`
	SignatureSlot                 uint64            `json:"signature_slot"`
	BlockRootProof                []string          `json:"block_root_proof"`
	BlockRootProofFinalizedHeader string            `json:"block_root_proof_finalized_header"`
}

type AttestationJSON struct {
	AggregationBits string              `json:"aggregation_bits"`
	Data            AttestationDataJSON `json:"data"`
	Signature       string              `json:"signature"`
}

type DepositDataJSON struct {
	Pubkey                string `json:"pubkey"`
	WithdrawalCredentials string `json:"withdrawal_credentials"`
	Amount                uint64 `json:"amount"`
	Signature             string `json:"signature"`
}

type VoluntaryExitJSON struct {
	Epoch          uint64 `json:"epoch"`
	ValidatorIndex uint64 `json:"validator_index"`
}

type DepositJSON struct {
	Proof []string        `json:"proof"`
	Data  DepositDataJSON `json:"data"`
}

func (p SyncCommitteePeriodPayload) ToJSON() SyncCommitteePeriodPayloadJSON {
	pubkeys := []string{}
	for _, pubkeyScale := range p.NextSyncCommittee.Pubkeys {
		pubkeys = append(pubkeys, bytesToHexString(pubkeyScale[:]))
	}

	nextSyncCommitteeBranch := []string{}
	for _, branch := range p.NextSyncCommitteeBranch {
		nextSyncCommitteeBranch = append(nextSyncCommitteeBranch, branch.Hex())
	}

	finalityBranch := []string{}
	for _, branch := range p.FinalityBranch {
		finalityBranch = append(finalityBranch, branch.Hex())
	}

	blockRootProof := []string{}
	for _, proof := range p.BlockRootProof {
		blockRootProof = append(blockRootProof, proof.Hex())
	}

	return SyncCommitteePeriodPayloadJSON{
		AttestedHeader: BeaconHeaderJSON{
			Slot:          uint64(p.AttestedHeader.Slot),
			ProposerIndex: uint64(p.AttestedHeader.ProposerIndex),
			ParentRoot:    p.AttestedHeader.ParentRoot.Hex(),
			StateRoot:     p.AttestedHeader.StateRoot.Hex(),
			BodyRoot:      p.AttestedHeader.BodyRoot.Hex(),
		},
		NextSyncCommittee: SyncCommitteeJSON{
			Pubkeys:         pubkeys,
			AggregatePubkey: bytesToHexString(p.NextSyncCommittee.AggregatePubkey[:]),
		},
		NextSyncCommitteeBranch: nextSyncCommitteeBranch,
		FinalizedHeader: BeaconHeaderJSON{
			Slot:          uint64(p.FinalizedHeader.Slot),
			ProposerIndex: uint64(p.FinalizedHeader.ProposerIndex),
			ParentRoot:    p.FinalizedHeader.ParentRoot.Hex(),
			StateRoot:     p.FinalizedHeader.StateRoot.Hex(),
			BodyRoot:      p.FinalizedHeader.BodyRoot.Hex(),
		},
		FinalityBranch: finalityBranch,
		SyncAggregate: SyncAggregateJSON{
			SyncCommitteeBits:      bytesToHexString(p.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(p.SyncAggregate.SyncCommitteeSignature),
		},
		SyncCommitteePeriod: uint64(p.SyncCommitteePeriod),
		SignatureSlot:       uint64(p.SignatureSlot),
		BlockRootsHash:      p.BlockRootsHash.Hex(),
		BlockRootProof:      blockRootProof,
	}
}

func (p FinalizedHeaderPayload) ToJSON() FinalizedHeaderPayloadJSON {
	fmt.Println(p.AttestedHeader.ProposerIndex)
	fmt.Println(p.FinalizedHeader.ProposerIndex)

	finalityBranch := []string{}
	for _, branch := range p.FinalityBranch {
		finalityBranch = append(finalityBranch, branch.Hex())
	}

	blockRootProof := []string{}
	for _, proof := range p.BlockRootProof {
		blockRootProof = append(blockRootProof, proof.Hex())
	}

	return FinalizedHeaderPayloadJSON{
		AttestedHeader: BeaconHeaderJSON{
			Slot:          uint64(p.AttestedHeader.Slot),
			ProposerIndex: uint64(p.AttestedHeader.ProposerIndex),
			ParentRoot:    p.AttestedHeader.ParentRoot.Hex(),
			StateRoot:     p.AttestedHeader.StateRoot.Hex(),
			BodyRoot:      p.AttestedHeader.BodyRoot.Hex(),
		},
		FinalizedHeader: BeaconHeaderJSON{
			Slot:          uint64(p.FinalizedHeader.Slot),
			ProposerIndex: uint64(p.FinalizedHeader.ProposerIndex),
			ParentRoot:    p.FinalizedHeader.ParentRoot.Hex(),
			StateRoot:     p.FinalizedHeader.StateRoot.Hex(),
			BodyRoot:      p.FinalizedHeader.BodyRoot.Hex(),
		},
		FinalityBranch: finalityBranch,
		SyncAggregate: SyncAggregateJSON{
			SyncCommitteeBits:      bytesToHexString(p.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(p.SyncAggregate.SyncCommitteeSignature),
		},
		SignatureSlot:  uint64(p.SignatureSlot),
		BlockRootsHash: p.BlockRootsHash.Hex(),
		BlockRootProof: blockRootProof,
	}
}

func (h HeaderUpdate) ToJSON() HeaderUpdatePayloadJSON {
	proposerSlashings := []ProposerSlashingJSON{}
	for _, proposerSlashing := range h.Block.Body.ProposerSlashings {
		proposerSlashings = append(proposerSlashings, ProposerSlashingJSON{
			SignedHeader1: SignedHeaderJSON{
				Message: BeaconHeaderJSON{
					Slot:          uint64(proposerSlashing.SignedHeader1.Message.Slot),
					ProposerIndex: uint64(proposerSlashing.SignedHeader1.Message.ProposerIndex),
					ParentRoot:    proposerSlashing.SignedHeader1.Message.ParentRoot.Hex(),
					StateRoot:     proposerSlashing.SignedHeader1.Message.StateRoot.Hex(),
					BodyRoot:      proposerSlashing.SignedHeader1.Message.BodyRoot.Hex(),
				},
				Signature: bytesToHexString(proposerSlashing.SignedHeader1.Signature),
			},
			SignedHeader2: SignedHeaderJSON{
				Message: BeaconHeaderJSON{
					Slot:          uint64(proposerSlashing.SignedHeader2.Message.Slot),
					ProposerIndex: uint64(proposerSlashing.SignedHeader2.Message.ProposerIndex),
					ParentRoot:    proposerSlashing.SignedHeader2.Message.ParentRoot.Hex(),
					StateRoot:     proposerSlashing.SignedHeader2.Message.StateRoot.Hex(),
					BodyRoot:      proposerSlashing.SignedHeader2.Message.BodyRoot.Hex(),
				},
				Signature: bytesToHexString(proposerSlashing.SignedHeader2.Signature),
			},
		})
	}

	attesterSlashings := []AttesterSlashingJSON{}
	for _, attesterSlashing := range h.Block.Body.AttesterSlashings {
		attesterSlashings = append(attesterSlashings, AttesterSlashingJSON{
			Attestation1: IndexedAttestationJSON{
				AttestingIndices: toUint64Array(attesterSlashing.Attestation1.AttestingIndices),
				Data: AttestationDataJSON{
					Slot:            uint64(attesterSlashing.Attestation1.Data.Slot),
					Index:           uint64(attesterSlashing.Attestation1.Data.Index),
					BeaconBlockRoot: attesterSlashing.Attestation1.Data.BeaconBlockRoot.Hex(),
					Source: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation1.Data.Source.Epoch),
						Root:  attesterSlashing.Attestation1.Data.Source.Root.Hex(),
					},
					Target: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation1.Data.Target.Epoch),
						Root:  attesterSlashing.Attestation1.Data.Target.Root.Hex(),
					},
				},
				Signature: bytesToHexString(attesterSlashing.Attestation1.Signature),
			},
			Attestation2: IndexedAttestationJSON{
				AttestingIndices: toUint64Array(attesterSlashing.Attestation2.AttestingIndices),
				Data: AttestationDataJSON{
					Slot:            uint64(attesterSlashing.Attestation2.Data.Slot),
					Index:           uint64(attesterSlashing.Attestation2.Data.Index),
					BeaconBlockRoot: attesterSlashing.Attestation2.Data.BeaconBlockRoot.Hex(),
					Source: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation2.Data.Source.Epoch),
						Root:  attesterSlashing.Attestation2.Data.Source.Root.Hex(),
					},
					Target: CheckpointJSON{
						Epoch: uint64(attesterSlashing.Attestation2.Data.Target.Epoch),
						Root:  attesterSlashing.Attestation2.Data.Target.Root.Hex(),
					},
				},
				Signature: bytesToHexString(attesterSlashing.Attestation2.Signature),
			},
		})
	}

	attestations := []AttestationJSON{}
	for _, attestation := range h.Block.Body.Attestations {
		attestations = append(attestations, AttestationJSON{
			AggregationBits: bytesToHexString(attestation.AggregationBits),
			Data: AttestationDataJSON{
				Slot:            uint64(attestation.Data.Slot),
				Index:           uint64(attestation.Data.Index),
				BeaconBlockRoot: attestation.Data.BeaconBlockRoot.Hex(),
				Source: CheckpointJSON{
					Epoch: uint64(attestation.Data.Source.Epoch),
					Root:  attestation.Data.Source.Root.Hex(),
				},
				Target: CheckpointJSON{
					Epoch: uint64(attestation.Data.Target.Epoch),
					Root:  attestation.Data.Target.Root.Hex(),
				},
			},
			Signature: bytesToHexString(attestation.Signature),
		})
	}

	deposits := []DepositJSON{}
	for _, deposit := range h.Block.Body.Deposits {
		deposits = append(deposits, DepositJSON{
			Proof: scaleBranchToString(deposit.Proof),
			Data: DepositDataJSON{
				Pubkey:                bytesToHexString(deposit.Data.Pubkey),
				WithdrawalCredentials: deposit.Data.WithdrawalCredentials.Hex(),
				Amount:                uint64(deposit.Data.Amount),
				Signature:             bytesToHexString(deposit.Data.Signature),
			},
		})
	}

	voluntaryExits := []VoluntaryExitJSON{}
	for _, voluntaryExit := range h.Block.Body.VoluntaryExits {
		voluntaryExits = append(voluntaryExits, VoluntaryExitJSON{
			Epoch:          uint64(voluntaryExit.Epoch),
			ValidatorIndex: uint64(voluntaryExit.ValidaterIndex),
		})
	}

	return HeaderUpdatePayloadJSON{
		Block: BlockJSON{
			Slot:          uint64(h.Block.Slot),
			ProposerIndex: uint64(h.Block.ProposerIndex),
			ParentRoot:    h.Block.ParentRoot.Hex(),
			StateRoot:     h.Block.StateRoot.Hex(),
			Body: BlockBodyJSON{
				RandaoReveal: bytesToHexString(h.Block.Body.RandaoReveal),
				Eth1Data: Eth1DataJSON{
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
				SyncAggregate: SyncAggregateJSON{
					SyncCommitteeBits:      bytesToHexString(h.Block.Body.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: bytesToHexString(h.Block.Body.SyncAggregate.SyncCommitteeSignature),
				},
				ExecutionPayload: ExecutionPayloadJSON{
					ParentHash:      h.Block.Body.ExecutionPayload.ParentHash.Hex(),
					FeeRecipient:    bytesToHexString(h.Block.Body.ExecutionPayload.FeeRecipient),
					StateRoot:       h.Block.Body.ExecutionPayload.StateRoot.Hex(),
					ReceiptsRoot:    h.Block.Body.ExecutionPayload.ReceiptsRoot.Hex(),
					LogsBloom:       bytesToHexString(h.Block.Body.ExecutionPayload.LogsBloom),
					PrevRandao:      h.Block.Body.ExecutionPayload.PrevRandao.Hex(),
					BlockNumber:     uint64(h.Block.Body.ExecutionPayload.BlockNumber),
					GasLimit:        uint64(h.Block.Body.ExecutionPayload.GasLimit),
					GasUsed:         uint64(h.Block.Body.ExecutionPayload.GasUsed),
					Timestamp:       uint64(h.Block.Body.ExecutionPayload.Timestamp),
					ExtraData:       bytesToHexString(h.Block.Body.ExecutionPayload.ExtraData),
					BaseFeePerGas:   h.Block.Body.ExecutionPayload.BaseFeePerGas.Uint64(),
					BlockHash:       h.Block.Body.ExecutionPayload.BlockHash.Hex(),
					TransactionRoot: h.Block.Body.ExecutionPayload.TransactionsRoot.Hex(),
				},
			},
		},
		SyncAggregate: SyncAggregateJSON{
			SyncCommitteeBits:      bytesToHexString(h.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: bytesToHexString(h.SyncAggregate.SyncCommitteeSignature),
		},
		SignatureSlot:                 uint64(h.SignatureSlot),
		BlockRootProof:                scaleBranchToString(h.BlockRootProof),
		BlockRootProofFinalizedHeader: h.BlockRootProofFinalizedHeader.Hex(),
	}
}

func (b *BeaconHeaderJSON) RemoveLeadingZeroHashes() {
	b.ParentRoot = removeLeadingZeroHash(b.ParentRoot)
	b.StateRoot = removeLeadingZeroHash(b.StateRoot)
	b.BodyRoot = removeLeadingZeroHash(b.BodyRoot)
}

func (s *SyncCommitteeJSON) RemoveLeadingZeroHashes() {
	for i, pubkey := range s.Pubkeys {
		s.Pubkeys[i] = removeLeadingZeroHash(pubkey)
	}

	s.AggregatePubkey = removeLeadingZeroHash(s.AggregatePubkey)
}

func (p *ProposerSlashingJSON) RemoveLeadingZeroHashes() {
	p.SignedHeader1.RemoveLeadingZeroHashes()
	p.SignedHeader2.RemoveLeadingZeroHashes()
}

func (a *AttesterSlashingJSON) RemoveLeadingZeroHashes() {
	a.Attestation1.RemoveLeadingZeroHashes()
	a.Attestation2.RemoveLeadingZeroHashes()
}

func (i *IndexedAttestationJSON) RemoveLeadingZeroHashes() {
	i.Data.RemoveLeadingZeroHashes()
	i.Signature = removeLeadingZeroHash(i.Signature)
}

func (a *AttestationDataJSON) RemoveLeadingZeroHashes() {
	a.BeaconBlockRoot = removeLeadingZeroHash(a.BeaconBlockRoot)
	a.Source.RemoveLeadingZeroHashes()
	a.Target.RemoveLeadingZeroHashes()
}

func (s *SignedHeaderJSON) RemoveLeadingZeroHashes() {
	s.Message.RemoveLeadingZeroHashes()
	s.Signature = removeLeadingZeroHash(s.Signature)
}

func (s *SyncAggregateJSON) RemoveLeadingZeroHashes() {
	s.SyncCommitteeBits = removeLeadingZeroHash(s.SyncCommitteeBits)
	s.SyncCommitteeSignature = removeLeadingZeroHash(s.SyncCommitteeSignature)
}

func (a *AttestationJSON) RemoveLeadingZeroHashes() {
	a.AggregationBits = removeLeadingZeroHash(a.AggregationBits)
	a.Data.RemoveLeadingZeroHashes()
	a.Signature = removeLeadingZeroHash(a.Signature)
}

func (c *CheckpointJSON) RemoveLeadingZeroHashes() {
	c.Root = removeLeadingZeroHash(c.Root)
}

func (d *DepositJSON) RemoveLeadingZeroHashes() {
	d.Data.Pubkey = removeLeadingZeroHash(d.Data.Pubkey)
	d.Data.Signature = removeLeadingZeroHash(d.Data.Signature)
	d.Data.WithdrawalCredentials = removeLeadingZeroHash(d.Data.WithdrawalCredentials)
}

func (b *BlockJSON) RemoveLeadingZeroHashes() {
	b.ParentRoot = removeLeadingZeroHash(b.ParentRoot)
	b.StateRoot = removeLeadingZeroHash(b.StateRoot)
	b.Body.RandaoReveal = removeLeadingZeroHash(b.Body.RandaoReveal)
	b.Body.Eth1Data.DepositRoot = removeLeadingZeroHash(b.Body.Eth1Data.DepositRoot)
	b.Body.Eth1Data.BlockHash = removeLeadingZeroHash(b.Body.Eth1Data.BlockHash)
	b.Body.Graffiti = removeLeadingZeroHash(b.Body.Graffiti)

	for i := range b.Body.ProposerSlashings {
		b.Body.ProposerSlashings[i].RemoveLeadingZeroHashes()
	}

	for i := range b.Body.AttesterSlashings {
		b.Body.AttesterSlashings[i].RemoveLeadingZeroHashes()
	}

	for i := range b.Body.Attestations {
		b.Body.Attestations[i].RemoveLeadingZeroHashes()
	}

	for i := range b.Body.Deposits {
		b.Body.Deposits[i].RemoveLeadingZeroHashes()
	}

	b.Body.SyncAggregate.RemoveLeadingZeroHashes()
	b.Body.ExecutionPayload.RemoveLeadingZeroHashes()
}

func (e *ExecutionPayloadJSON) RemoveLeadingZeroHashes() {
	e.ParentHash = removeLeadingZeroHash(e.ParentHash)
	e.FeeRecipient = removeLeadingZeroHash(e.FeeRecipient)
	e.StateRoot = removeLeadingZeroHash(e.StateRoot)
	e.ReceiptsRoot = removeLeadingZeroHash(e.ReceiptsRoot)
	e.LogsBloom = removeLeadingZeroHash(e.LogsBloom)
	e.PrevRandao = removeLeadingZeroHash(e.PrevRandao)
	e.ExtraData = removeLeadingZeroHash(e.ExtraData)
	e.BlockHash = removeLeadingZeroHash(e.BlockHash)
	e.TransactionRoot = removeLeadingZeroHash(e.TransactionRoot)
}

func (i *InitialSync) RemoveLeadingZeroHashes() {
	i.Header.RemoveLeadingZeroHashes()
	i.CurrentSyncCommittee.RemoveLeadingZeroHashes()

	for k, branch := range i.CurrentSyncCommitteeBranch {
		i.CurrentSyncCommitteeBranch[k] = removeLeadingZeroHash(branch)
	}

	i.ValidatorsRoot = removeLeadingZeroHash(i.ValidatorsRoot)
}

func (s *SyncCommitteePeriodPayloadJSON) RemoveLeadingZeroHashes() {
	s.AttestedHeader.RemoveLeadingZeroHashes()
	s.NextSyncCommittee.RemoveLeadingZeroHashes()
	s.NextSyncCommitteeBranch = removeLeadingZeroHashForSlice(s.NextSyncCommitteeBranch)
	s.FinalizedHeader.RemoveLeadingZeroHashes()
	s.FinalityBranch = removeLeadingZeroHashForSlice(s.FinalityBranch)
	s.SyncAggregate.RemoveLeadingZeroHashes()
	s.BlockRootsHash = removeLeadingZeroHash(s.BlockRootsHash)
	s.BlockRootProof = removeLeadingZeroHashForSlice(s.BlockRootProof)
}

func (f *FinalizedHeaderPayloadJSON) RemoveLeadingZeroHashes() {
	f.AttestedHeader.RemoveLeadingZeroHashes()
	f.FinalizedHeader.RemoveLeadingZeroHashes()
	f.FinalityBranch = removeLeadingZeroHashForSlice(f.FinalityBranch)
	f.SyncAggregate.RemoveLeadingZeroHashes()
	f.BlockRootsHash = removeLeadingZeroHash(f.BlockRootsHash)
	f.BlockRootProof = removeLeadingZeroHashForSlice(f.BlockRootProof)
}

func (h *HeaderUpdatePayloadJSON) RemoveLeadingZeroHashes() {
	h.Block.RemoveLeadingZeroHashes()
	h.SyncAggregate.RemoveLeadingZeroHashes()
	h.BlockRootProof = removeLeadingZeroHashForSlice(h.BlockRootProof)
	h.BlockRootProofFinalizedHeader = removeLeadingZeroHash(h.BlockRootProofFinalizedHeader)
}

func removeLeadingZeroHashForSlice(s []string) []string {
	result := make([]string, len(s))

	for i, item := range s {
		result[i] = removeLeadingZeroHash(item)
	}
	return result
}

func removeLeadingZeroHash(s string) string {
	return strings.Replace(s, "0x", "", 1)
}
