package api

import (
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

func (a AttesterSlashingResponse) ToFastSSZElectra() (*state.AttesterSlashingElectra, error) {
	attestation1, err := a.Attestation1.ToFastSSZElectra()
	if err != nil {
		return nil, err
	}

	attestation2, err := a.Attestation2.ToFastSSZElectra()
	if err != nil {
		return nil, err
	}

	return &state.AttesterSlashingElectra{
		Attestation1: attestation1,
		Attestation2: attestation2,
	}, nil
}

func (i IndexedAttestationResponse) ToFastSSZElectra() (*state.IndexedAttestationElectra, error) {
	data, err := i.Data.ToFastSSZ()
	if err != nil {
		return nil, err
	}

	attestationIndexes := []uint64{}
	for _, index := range i.AttestingIndices {
		indexInt, err := util.ToUint64(index)
		if err != nil {
			return nil, err
		}

		attestationIndexes = append(attestationIndexes, indexInt)
	}

	signature, err := util.HexStringToByteArray(i.Signature)
	if err != nil {
		return nil, err
	}

	return &state.IndexedAttestationElectra{
		AttestationIndices: attestationIndexes,
		Data:               data,
		Signature:          signature,
	}, nil
}

func (a AttestationResponse) ToFastSSZElectra() (*state.AttestationElectra, error) {
	data, err := a.Data.ToFastSSZ()
	if err != nil {
		return nil, err
	}

	aggregationBits, err := util.HexStringToByteArray(a.AggregationBits)
	if err != nil {
		return nil, err
	}

	signature, err := util.HexStringTo96Bytes(a.Signature)
	if err != nil {
		return nil, err
	}

	committeeBits, err := util.HexStringToByteArray(a.CommitteeBits)
	if err != nil {
		return nil, err
	}

	return &state.AttestationElectra{
		AggregationBits: aggregationBits,
		Data:            data,
		Signature:       signature,
		CommitteeBits:   committeeBits,
	}, nil
}
