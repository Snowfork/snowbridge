// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/common"

	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"golang.org/x/sync/errgroup"
)

type Message interface{}

// Message from Substrate
type SubstrateOutboundMessage struct {
	ChannelID      substrate.ChannelID
	CommitmentHash types.H256
	Commitment     []substrate.CommitmentMessage
}

// Message from ethereum
type EthereumOutboundMessage substrate.Message

// Message from Parachain
type NewSignatureCommitmentMessage struct {
	Payload                       [32]byte
	ValidatorClaimsBitfield       *big.Int
	ValidatorSignatureCommitment  []byte
	ValidatorPublicKey            common.Address
	ValidatorPublicKeyMerkleProof [][32]byte
}

type Header struct {
	HeaderData interface{}
	ProofData  interface{}
}

type Init interface{}

type Chain interface {
	Name() string
	Start(ctx context.Context, eg *errgroup.Group, initOut chan<- Init, initIn <-chan Init) error
	Stop()
	SetReceiver(messages <-chan []Message, headers <-chan Header) error
	SetSender(messages chan<- []Message, headers chan<- Header) error
}
