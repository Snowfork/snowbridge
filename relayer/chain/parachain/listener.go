// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package parachain

import (
	"context"
	"crypto/sha256"
	"fmt"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/common"
	// "github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/sirupsen/logrus"
	merkletree "github.com/wealdtech/go-merkletree"
	"golang.org/x/sync/errgroup"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/snowfork/go-substrate-rpc-client/v2/types"
	"github.com/snowfork/polkadot-ethereum/relayer/chain"
	chainTypes "github.com/snowfork/polkadot-ethereum/relayer/substrate"
)

type Listener struct {
	config   *Config
	conn     *Connection
	messages chan<- []chain.Message
	log      *logrus.Entry
}

func NewListener(config *Config, conn *Connection, messages chan<- []chain.Message, log *logrus.Entry) *Listener {
	return &Listener{
		config:   config,
		conn:     conn,
		messages: messages,
		log:      log,
	}
}

func (li *Listener) Start(ctx context.Context, eg *errgroup.Group) error {
	li.log.Infof("\n\nParachain listener empty start function\n\n")

	eg.Go(func() error {
		return li.subBeefyJustifications(ctx)
	})

	return nil
}

func (li *Listener) onDone(ctx context.Context) error {
	li.log.Info("Shutting down listener...")
	close(li.messages)
	return ctx.Err()
}

func (li *Listener) subBeefyJustifications(ctx context.Context) error {
	ch := make(chan interface{})

	sub, err := li.conn.api.Client.Subscribe(context.Background(), "beefy", "subscribeJustifications", "unsubscribeJustifications", "justifications", ch)
	if err != nil {
		panic(err)
	}
	defer sub.Unsubscribe()

	// timeout := time.After(40 * time.Second)
	received := 0

	for {
		select {
		case <-ctx.Done():
			return li.onDone(ctx)
		case msg := <-ch:

			// TODO: var messages []chainTypes.BeefyMessage
			signedCommitment := &SignedCommitment{}
			err := types.DecodeFromHexString(msg.(string), signedCommitment)
			if err != nil {
				li.log.WithError(err).Error("Faild to decode beefy commitment messages")
			}

			received++
			li.log.Info("--------------------------------------------------------------")
			li.log.Info("BEEFY commitment received: ", received)

			if len(signedCommitment.Signatures) == 0 {
				li.log.Info("BEEFY commitment has no signatures, skipping...")
				continue
			}

			// Construct BEEFY merkle tree
			beefyValidatorAddresses := []string{
				"0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b",
				"0x25451A4de12dcCc2D166922fA938E900fCc4ED24",
			}

			var beefyTreeData [][]byte
			for _, valAddr := range beefyValidatorAddresses {
				h := sha256.New()
				if _, err := h.Write([]byte(valAddr)); err != nil {
					li.log.Info("err:", err)
				}
				hashedData := h.Sum(nil)
				beefyTreeData = append(beefyTreeData, hashedData)
			}

			// Create the tree
			beefyMerkleTree, err := merkletree.New(beefyTreeData)
			if err != nil {
				li.log.Info("err:", err)
			}

			// Fetch the root hash of the tree
			root := beefyMerkleTree.Root()

			// Generate a proof
			sig0ProofData := beefyTreeData[0]
			sig0Proof, err := beefyMerkleTree.GenerateProof(sig0ProofData)
			if err != nil {
				li.log.Info("err:", err)
			}

			// Verify the proof
			verified, err := merkletree.VerifyProof(sig0ProofData, sig0Proof, root)
			if err != nil {
				li.log.Info("err:", err)
			}
			if !verified {
				panic("failed to verify proof")
			}

			sig0ProofContents := make([][32]byte, len(sig0Proof.Hashes))
			for i, hash := range sig0Proof.Hashes {
				var hash32Byte [32]byte
				copy(hash32Byte[:], hash)
				sig0ProofContents[i] = hash32Byte
			}

			// Build hashed commitment
			h := sha256.New()
			commitmentBytes := []byte(fmt.Sprintf("%v", signedCommitment.Commitment))
			if _, err := h.Write(commitmentBytes); err != nil {
				li.log.Info("err:", err)
			}
			hashedCommitment := h.Sum(nil)
			var hashedCommitment32Byte [32]byte
			copy(hashedCommitment32Byte[:], hashedCommitment)

			// // Update signature format (Polkadot uses recovery IDs 0 or 1, Eth uses 27 or 28, so we need to add 27)
			// recIdIncrement := big.NewInt(27)
			ok, sig0 := signedCommitment.Signatures[0].Unwrap()
			if !ok {
				li.log.Info("err:", err)
			}

			// sig0HexStr := hexutil.Encode(sig0[:])             // bytes -> 0x string
			// recoveryId0, err := hexutil.DecodeBig(sig0HexStr) // 0x string -> big.Int
			// if err != nil {
			// 	li.log.Info("err:", err)
			// }
			// incrementedRecoveryId0 := recoveryId0.Add(recoveryId0, recIdIncrement)
			// newRecoveryId0 := hexutil.EncodeBig(incrementedRecoveryId0) // big.Int -> 0x string
			// newRecoveryId0Bytes, err := hexutil.Decode(newRecoveryId0)  // 0x string -> []byte
			// if err != nil {
			// 	li.log.Info("err:", err)
			// }
			// valSigCommitment := append(sig0[:], newRecoveryId0Bytes...)

			// TODO: increment recovery ID properly
			valSigCommitment := sig0[:]

			sig0ValAddr := common.HexToAddress(beefyValidatorAddresses[0])

			message := chain.NewSignatureCommitmentMessage{
				Payload:                       hashedCommitment32Byte,
				ValidatorClaimsBitfield:       big.NewInt(123), // TODO: add bitfield stuff properly
				ValidatorSignatureCommitment:  valSigCommitment,
				ValidatorPublicKey:            sig0ValAddr,
				ValidatorPublicKeyMerkleProof: sig0ProofContents,
			}

			li.log.Info("Sending BEEFY commitment message to Ethereum messages chan")
			li.messages <- []chain.Message{message}
		}
	}
}

func sleep(ctx context.Context, delay time.Duration) {
	select {
	case <-ctx.Done():
	case <-time.After(delay):
	}
}

func getAuxiliaryDigestItem(digest types.Digest) (*chainTypes.AuxiliaryDigestItem, error) {
	for _, digestItem := range digest {
		if digestItem.IsOther {
			var auxDigestItem chainTypes.AuxiliaryDigestItem
			err := types.DecodeFromBytes(digestItem.AsOther, &auxDigestItem)
			if err != nil {
				return nil, err
			}
			return &auxDigestItem, nil
		}
	}
	return nil, nil
}

// TODO: use these types from GSRPC's types/beefy.go once it's merged/published

// Commitment is a beefy commitment
type Commitment struct {
	Payload        types.H256
	BlockNumber    types.BlockNumber
	ValidatorSetID types.U64
}

// SignedCommitment is a beefy commitment with optional signatures from the set of validators
type SignedCommitment struct {
	Commitment Commitment
	Signatures []OptionBeefySignature
}

// BeefySignature is a beefy signature
type BeefySignature [65]byte

// OptionBeefySignature is a structure that can store a BeefySignature or a missing value
type OptionBeefySignature struct {
	option
	value BeefySignature
}

// NewOptionBeefySignature creates an OptionBeefySignature with a value
func NewOptionBeefySignature(value BeefySignature) OptionBeefySignature {
	return OptionBeefySignature{option{true}, value}
}

// NewOptionBeefySignatureEmpty creates an OptionBeefySignature without a value
func NewOptionBeefySignatureEmpty() OptionBeefySignature {
	return OptionBeefySignature{option: option{false}}
}

func (o OptionBeefySignature) Encode(encoder scale.Encoder) error {
	return encoder.EncodeOption(o.hasValue, o.value)
}

func (o *OptionBeefySignature) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.hasValue, &o.value)
}

// SetSome sets a value
func (o *OptionBeefySignature) SetSome(value BeefySignature) {
	o.hasValue = true
	o.value = value
}

// SetNone removes a value and marks it as missing
func (o *OptionBeefySignature) SetNone() {
	o.hasValue = false
	o.value = BeefySignature{}
}

// Unwrap returns a flag that indicates whether a value is present and the stored value
func (o OptionBeefySignature) Unwrap() (ok bool, value BeefySignature) {
	return o.hasValue, o.value
}

type option struct {
	hasValue bool
}

// IsNone returns true if the value is missing
func (o option) IsNone() bool {
	return !o.hasValue
}

// IsNone returns true if a value is present
func (o option) IsSome() bool {
	return o.hasValue
}
