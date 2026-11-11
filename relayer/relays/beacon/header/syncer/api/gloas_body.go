package api

// Construction of a Gloas BeaconBlockBody from a beacon node's JSON block response.
//
// The body is needed only for its hash tree root and for the execution commitment proof, so
// the operation lists are reduced to their roots here rather than kept as decoded objects.

import (
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"

	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

func decodeHex(field, s string) ([]byte, error) {
	b, err := hex.DecodeString(strings.TrimPrefix(s, "0x"))
	if err != nil {
		return nil, fmt.Errorf("decode %s: %w", field, err)
	}
	return b, nil
}

func decodeUint(field, s string) (uint64, error) {
	v, err := strconv.ParseUint(s, 10, 64)
	if err != nil {
		return 0, fmt.Errorf("decode %s: %w", field, err)
	}
	return v, nil
}

// BuildGloasBody assembles a Gloas body from a block response.
//
// It fails rather than guesses. A block carrying an operation whose merkleization is not
// implemented returns ErrUnsupportedBodyList, because a body root computed from a stand-in
// would yield a proof the light client rejects without saying why.
func BuildGloasBody(b *BeaconBlockResponseBody) (*state.BeaconBlockBodyGloas, error) {
	if b.SignedExecutionPayloadBid == nil {
		return nil, fmt.Errorf("block has no signed_execution_payload_bid; not a gloas block")
	}
	for field, n := range map[string]int{
		"proposer_slashings":       len(b.ProposerSlashings),
		"attester_slashings":       len(b.AttesterSlashings),
		"deposits":                 len(b.Deposits),
		"voluntary_exits":          len(b.VoluntaryExits),
		"bls_to_execution_changes": len(b.BlsToExecutionChanges),
	} {
		if n > 0 {
			return nil, state.ErrUnsupportedBodyList{Field: field}
		}
	}

	attestations, err := buildAttestations(b.Attestations)
	if err != nil {
		return nil, err
	}
	attestationsRoot, err := state.AttestationsRoot(attestations)
	if err != nil {
		return nil, fmt.Errorf("merkleize attestations: %w", err)
	}

	payloadAttestations, err := buildPayloadAttestations(b.PayloadAttestations)
	if err != nil {
		return nil, err
	}
	payloadAttestationsRoot, err := state.PayloadAttestationsRoot(payloadAttestations)
	if err != nil {
		return nil, fmt.Errorf("merkleize payload attestations: %w", err)
	}

	requests := &state.ExecutionRequestsGloas{}
	if r := b.ParentExecutionRequests; r != nil {
		requests = &state.ExecutionRequestsGloas{
			Deposits:        len(r.Deposits),
			Withdrawals:     len(r.Withdrawals),
			Consolidations:  len(r.Consolidations),
			BuilderDeposits: len(r.BuilderDeposits),
			BuilderExits:    len(r.BuilderExits),
		}
	}
	requestsRoot, err := requests.HashTreeRoot()
	if err != nil {
		return nil, err
	}

	bid, err := buildBid(b)
	if err != nil {
		return nil, err
	}
	eth1Data, err := buildEth1Data(b)
	if err != nil {
		return nil, err
	}

	randao, err := decodeHex("randao_reveal", b.RandaoReveal)
	if err != nil {
		return nil, err
	}
	graffiti, err := decodeHex("graffiti", b.Graffiti)
	if err != nil {
		return nil, err
	}
	syncBits, err := decodeHex("sync_committee_bits", b.SyncAggregate.SyncCommitteeBits)
	if err != nil {
		return nil, err
	}
	syncSig, err := decodeHex("sync_committee_signature", b.SyncAggregate.SyncCommitteeSignature)
	if err != nil {
		return nil, err
	}

	empty := state.EmptyProgressiveListRoot()
	return &state.BeaconBlockBodyGloas{
		RandaoReveal:              randao,
		Eth1Data:                  eth1Data,
		Graffiti:                  graffiti,
		ProposerSlashingsRoot:     empty,
		AttesterSlashingsRoot:     empty,
		AttestationsRoot:          attestationsRoot,
		DepositsRoot:              empty,
		VoluntaryExitsRoot:        empty,
		SyncAggregateRoot:         state.SyncAggregateRoot(syncBits, syncSig),
		BlsToExecutionChangesRoot: empty,
		SignedExecutionPayloadBid: bid,
		PayloadAttestationsRoot:   payloadAttestationsRoot,
		ParentExecutionRequests:   requestsRoot,
	}, nil
}

func buildCheckpoint(c CheckpointResponse) (*state.Checkpoint, error) {
	epoch, err := decodeUint("checkpoint.epoch", c.Epoch)
	if err != nil {
		return nil, err
	}
	root, err := decodeHex("checkpoint.root", c.Root)
	if err != nil {
		return nil, err
	}
	return &state.Checkpoint{Epoch: epoch, Root: root}, nil
}

func buildAttestations(list []AttestationResponse) ([]*state.AttestationGloas, error) {
	out := make([]*state.AttestationGloas, len(list))
	for i, a := range list {
		bits, err := decodeHex("aggregation_bits", a.AggregationBits)
		if err != nil {
			return nil, err
		}
		committeeBits, err := decodeHex("committee_bits", a.CommitteeBits)
		if err != nil {
			return nil, err
		}
		sig, err := decodeHex("attestation.signature", a.Signature)
		if err != nil {
			return nil, err
		}
		slot, err := decodeUint("attestation.slot", a.Data.Slot)
		if err != nil {
			return nil, err
		}
		index, err := decodeUint("attestation.index", a.Data.Index)
		if err != nil {
			return nil, err
		}
		blockRoot, err := decodeHex("attestation.beacon_block_root", a.Data.BeaconBlockRoot)
		if err != nil {
			return nil, err
		}
		source, err := buildCheckpoint(a.Data.Source)
		if err != nil {
			return nil, err
		}
		target, err := buildCheckpoint(a.Data.Target)
		if err != nil {
			return nil, err
		}
		out[i] = &state.AttestationGloas{
			AggregationBits: bits,
			Data: &state.AttestationDataGloas{
				Slot:            slot,
				Index:           index,
				BeaconBlockRoot: blockRoot,
				Source:          source,
				Target:          target,
			},
			Signature:     sig,
			CommitteeBits: committeeBits,
		}
	}
	return out, nil
}

func buildPayloadAttestations(list []PayloadAttestationResponse) ([]*state.PayloadAttestationGloas, error) {
	out := make([]*state.PayloadAttestationGloas, len(list))
	for i, p := range list {
		bits, err := decodeHex("payload_attestation.aggregation_bits", p.AggregationBits)
		if err != nil {
			return nil, err
		}
		sig, err := decodeHex("payload_attestation.signature", p.Signature)
		if err != nil {
			return nil, err
		}
		blockRoot, err := decodeHex("payload_attestation.beacon_block_root", p.Data.BeaconBlockRoot)
		if err != nil {
			return nil, err
		}
		slot, err := decodeUint("payload_attestation.slot", p.Data.Slot)
		if err != nil {
			return nil, err
		}
		out[i] = &state.PayloadAttestationGloas{
			AggregationBits: bits,
			Data: &state.PayloadAttestationDataGloas{
				BeaconBlockRoot:   blockRoot,
				Slot:              slot,
				PayloadPresent:    p.Data.PayloadPresent,
				BlobDataAvailable: p.Data.BlobDataAvailable,
			},
			Signature: sig,
		}
	}
	return out, nil
}

func buildEth1Data(b *BeaconBlockResponseBody) (*state.Eth1Data, error) {
	depositRoot, err := decodeHex("eth1_data.deposit_root", b.Eth1Data.DepositRoot)
	if err != nil {
		return nil, err
	}
	depositCount, err := decodeUint("eth1_data.deposit_count", b.Eth1Data.DepositCount)
	if err != nil {
		return nil, err
	}
	blockHash, err := decodeHex("eth1_data.block_hash", b.Eth1Data.BlockHash)
	if err != nil {
		return nil, err
	}
	return &state.Eth1Data{DepositRoot: depositRoot, DepositCount: depositCount, BlockHash: blockHash}, nil
}

func buildBid(b *BeaconBlockResponseBody) (*state.SignedExecutionPayloadBidGloas, error) {
	m := b.SignedExecutionPayloadBid.Message
	bytesFields := map[string]*string{
		"bid.parent_block_hash":       &m.ParentBlockHash,
		"bid.parent_block_root":       &m.ParentBlockRoot,
		"bid.block_hash":              &m.BlockHash,
		"bid.prev_randao":             &m.PrevRandao,
		"bid.fee_recipient":           &m.FeeRecipient,
		"bid.execution_requests_root": &m.ExecutionRequestsRoot,
	}
	decoded := make(map[string][]byte, len(bytesFields))
	for name, src := range bytesFields {
		v, err := decodeHex(name, *src)
		if err != nil {
			return nil, err
		}
		decoded[name] = v
	}
	uintFields := map[string]*string{
		"bid.gas_limit":         &m.GasLimit,
		"bid.builder_index":     &m.BuilderIndex,
		"bid.slot":              &m.Slot,
		"bid.value":             &m.Value,
		"bid.execution_payment": &m.ExecutionPayment,
	}
	nums := make(map[string]uint64, len(uintFields))
	for name, src := range uintFields {
		v, err := decodeUint(name, *src)
		if err != nil {
			return nil, err
		}
		nums[name] = v
	}
	commitments := make([][]byte, len(m.BlobKzgCommitments))
	for i, c := range m.BlobKzgCommitments {
		v, err := decodeHex("bid.blob_kzg_commitments", c)
		if err != nil {
			return nil, err
		}
		commitments[i] = v
	}
	sig, err := decodeHex("bid.signature", b.SignedExecutionPayloadBid.Signature)
	if err != nil {
		return nil, err
	}
	return &state.SignedExecutionPayloadBidGloas{
		Message: &state.ExecutionPayloadBid{
			ParentBlockHash:       decoded["bid.parent_block_hash"],
			ParentBlockRoot:       decoded["bid.parent_block_root"],
			BlockHash:             decoded["bid.block_hash"],
			PrevRandao:            decoded["bid.prev_randao"],
			FeeRecipient:          decoded["bid.fee_recipient"],
			GasLimit:              nums["bid.gas_limit"],
			BuilderIndex:          nums["bid.builder_index"],
			Slot:                  nums["bid.slot"],
			Value:                 nums["bid.value"],
			ExecutionPayment:      nums["bid.execution_payment"],
			BlobKzgCommitments:    commitments,
			ExecutionRequestsRoot: decoded["bid.execution_requests_root"],
		},
		Signature: sig,
	}, nil
}
