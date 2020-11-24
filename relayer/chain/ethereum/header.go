// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"fmt"
	"math/big"

	types "github.com/centrifuge/go-substrate-rpc-client/types"
	etypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/sirupsen/logrus"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
)

type Header struct {
	ParentHash       types.H256
	Timestamp        types.U64
	Number           types.U64
	Author           types.H160
	TransactionsRoot types.H256
	OmmersHash       types.H256
	ExtraData        types.Bytes
	StateRoot        types.H256
	ReceiptsRoot     types.H256
	LogsBloom        types.Bytes256
	GasUsed          types.U256
	GasLimit         types.U256
	Difficulty       types.U256
	Seal             []types.Bytes
}

type DoubleNodeWithMerkleProof struct {
	DagNodes [2]types.H512
	Proof    [][16]byte
}

func MakeHeaderFromEthHeader(gethheader *etypes.Header, log *logrus.Entry) (*chain.Header, error) {

	// Convert Geth types to their Substrate Go client counterparts that match our node
	if !gethheader.Number.IsUint64() {
		return nil, fmt.Errorf("gethheader.Number is not uint64")
	}

	var gasUsed, gasLimit big.Int
	gasUsed.SetUint64(gethheader.GasUsed)
	gasLimit.SetUint64(gethheader.GasLimit)

	var bloomBytes [256]byte
	copy(bloomBytes[:], gethheader.Bloom.Bytes())

	mixHashRLP, err := rlp.EncodeToBytes(gethheader.MixDigest)
	if err != nil {
		return nil, err
	}

	nonceRLP, err := rlp.EncodeToBytes(gethheader.Nonce)
	if err != nil {
		return nil, err
	}

	header := Header{
		ParentHash:       types.NewH256(gethheader.ParentHash.Bytes()),
		Timestamp:        types.NewU64(gethheader.Time),
		Number:           types.NewU64(gethheader.Number.Uint64()),
		Author:           types.NewH160(gethheader.Coinbase.Bytes()),
		TransactionsRoot: types.NewH256(gethheader.TxHash.Bytes()),
		OmmersHash:       types.NewH256(gethheader.UncleHash.Bytes()),
		ExtraData:        types.NewBytes(gethheader.Extra),
		StateRoot:        types.NewH256(gethheader.Root.Bytes()),
		ReceiptsRoot:     types.NewH256(gethheader.ReceiptHash.Bytes()),
		LogsBloom:        types.NewBytes256(bloomBytes),
		GasUsed:          types.NewU256(gasUsed),
		GasLimit:         types.NewU256(gasLimit),
		Difficulty:       types.NewU256(*gethheader.Difficulty),
		Seal:             []types.Bytes{mixHashRLP, nonceRLP},
	}

	log.WithFields(logrus.Fields{
		"blockNumber": gethheader.Number,
	}).Debug("Generated header from Ethereum header")

	// TODO: integrate ethashproof and plug data in here
	var proofData []DoubleNodeWithMerkleProof

	return &chain.Header{HeaderData: header, ProofData: proofData}, nil
}
