package beefy

import (
	"encoding/json"

	"github.com/ethereum/go-ethereum/common"
	"github.com/jinzhu/gorm"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/crypto/merkle"
)

type Status int

const (
	CommitmentWitnessed            Status = iota // 0
	InitialVerificationTxSent      Status = iota // 1
	InitialVerificationTxConfirmed Status = iota // 2
	ReadyToComplete                Status = iota // 3
	CompleteVerificationTxSent     Status = iota // 4
)

type TaskRecord struct {
	gorm.Model
	ValidatorsBytes       []byte
	SignedCommitmentBytes []byte
	ProofBytes            []byte
	ValidationID          int64
	Status                Status
	InitialVerificationTx common.Hash
	CompleteOnBlock       uint64
	FinalVerificationTx   common.Hash
}

type Task struct {
	TaskRecord
	Validators       []common.Address
	SignedCommitment types.SignedCommitment
	Proof            merkle.SimplifiedMMRProof
}

func (r *TaskRecord) Thaw() (*Task, error) {
	var validators []common.Address
	if err := json.Unmarshal(r.ValidatorsBytes, &validators); err != nil {
		return nil, err
	}

	var signedCommitment types.SignedCommitment
	if err := types.DecodeFromBytes(r.SignedCommitmentBytes, &signedCommitment); err != nil {
		return nil, err
	}

	var proof merkle.SimplifiedMMRProof
	if err := types.DecodeFromBytes(r.ProofBytes, &proof); err != nil {
		return nil, err
	}

	return &Task{
		TaskRecord:       *r,
		Validators:       validators,
		SignedCommitment: signedCommitment,
		Proof:            proof,
	}, nil
}

func (t *Task) Freeze() (*TaskRecord, error) {
	signedCommitmentBytes, err := types.EncodeToBytes(t.SignedCommitment)
	if err != nil {
		return nil, err
	}

	validatorsBytes, err := json.Marshal(t.Validators)
	if err != nil {
		return nil, err
	}

	proofBytes, err := types.EncodeToBytes(t.Proof)
	if err != nil {
		return nil, err
	}

	record := t.TaskRecord
	record.ValidatorsBytes = validatorsBytes
	record.SignedCommitmentBytes = signedCommitmentBytes
	record.ProofBytes = proofBytes

	return &record, nil
}
