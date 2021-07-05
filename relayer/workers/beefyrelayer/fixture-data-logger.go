package beefyrelayer

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"math/big"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/sirupsen/logrus"
	gsrpcTypes "github.com/snowfork/go-substrate-rpc-client/v3/types"

	"github.com/snowfork/polkadot-ethereum/relayer/workers/beefyrelayer/store"
)

type BeefyLightClientCommitmentLog struct {
	Payload        string
	BlockNumber    uint64
	ValidatorSetId uint32
}

type BeefyLightClientValidatorProofLog struct {
	Signatures            []string
	Positions             []*big.Int
	PublicKeys            []common.Address
	PublicKeyMerkleProofs [][]string
}

type BeefyLightClientBeefyMMRLeafLog struct {
	ParentNumber         uint32
	ParentHash           string
	ParachainHeadsRoot   string
	NextAuthoritySetId   uint64
	NextAuthoritySetLen  uint32
	NextAuthoritySetRoot string
}

type CompleteSignatureCommitmentTxInput struct {
	Id             *big.Int
	Commitment     BeefyLightClientCommitmentLog
	ValidatorProof BeefyLightClientValidatorProofLog
	LatestMMRLeaf  BeefyLightClientBeefyMMRLeafLog
	MMRProofItems  []string
}

func (wr *BeefyEthereumWriter) LogBeefyFixtureDataAll(
	msg store.CompleteSignatureCommitmentMessage, info store.BeefyRelayInfo) error {

	var latestMMRProof gsrpcTypes.GenerateMMRProofResponse
	gsrpcTypes.DecodeFromBytes(info.SerializedLatestMMRProof, &latestMMRProof)

	var hasher Keccak256

	bytesEncodedMMRLeaf, _ := gsrpcTypes.EncodeToBytes(msg.LatestMMRLeaf)

	// Leaf is double encoded
	hexEncodedLeaf, _ := gsrpcTypes.EncodeToHexString(bytesEncodedMMRLeaf)
	bytesEncodedLeaf, _ := gsrpcTypes.EncodeToBytes(bytesEncodedMMRLeaf)

	hashedLeaf := "0x" + hex.EncodeToString(hasher.Hash(bytesEncodedLeaf))

	var mmrProofItems []string
	for _, item := range msg.MMRProofItems {
		hex := "0x" + hex.EncodeToString(item[:])
		mmrProofItems = append(mmrProofItems, hex)
	}

	var signatures []string
	for _, item := range msg.Signatures {
		hex := hex.EncodeToString(item)
		signatures = append(signatures, "0x"+hex)
	}

	var pubKeyMerkleProofs [][]string
	for _, pubkeyProof := range msg.ValidatorPublicKeyMerkleProofs {
		var pubkeyProofS []string
		for _, item := range pubkeyProof {
			hex := "0x" + hex.EncodeToString(item[:])
			pubkeyProofS = append(pubkeyProofS, hex)
		}
		pubKeyMerkleProofs = append(pubKeyMerkleProofs, pubkeyProofS)
	}

	input := &CompleteSignatureCommitmentTxInput{
		Id: msg.ID,
		Commitment: BeefyLightClientCommitmentLog{
			Payload:        hex.EncodeToString(msg.Commitment.Payload[:]),
			BlockNumber:    msg.Commitment.BlockNumber,
			ValidatorSetId: msg.Commitment.ValidatorSetId,
		},
		ValidatorProof: BeefyLightClientValidatorProofLog{
			Signatures:            signatures,
			Positions:             msg.ValidatorPositions,
			PublicKeys:            msg.ValidatorPublicKeys,
			PublicKeyMerkleProofs: pubKeyMerkleProofs,
		},
		LatestMMRLeaf: BeefyLightClientBeefyMMRLeafLog{
			ParentNumber:         msg.LatestMMRLeaf.ParentNumber,
			ParentHash:           hex.EncodeToString(msg.LatestMMRLeaf.ParentHash[:]),
			ParachainHeadsRoot:   hex.EncodeToString(msg.LatestMMRLeaf.ParachainHeadsRoot[:]),
			NextAuthoritySetId:   msg.LatestMMRLeaf.NextAuthoritySetId,
			NextAuthoritySetLen:  msg.LatestMMRLeaf.NextAuthoritySetLen,
			NextAuthoritySetRoot: hex.EncodeToString(msg.LatestMMRLeaf.NextAuthoritySetRoot[:]),
		},
		MMRProofItems: mmrProofItems,
	}
	b, err := json.Marshal(input)
	if err != nil {
		return err
	}

	wr.log.WithFields(logrus.Fields{
		"json":           string(b),
		"hexEncodedLeaf": hexEncodedLeaf,
		"hashedLeaf":     hashedLeaf,
		"mmrProofItems":  mmrProofItems,
	}).Info("Complete Signature Commitment transaction submitted")

	return nil
}

// Keccak256 is the Keccak256 hashing method
type Keccak256 struct{}

// New creates a new Keccak256 hashing method
func New() *Keccak256 {
	return &Keccak256{}
}

// Hash generates a Keccak256 hash from a byte array
func (h *Keccak256) Hash(data []byte) []byte {
	hash := crypto.Keccak256(data)
	return hash[:]
}

func (wr *BeefyEthereumWriter) GetFailingMessage(client ethclient.Client, hash common.Hash) (string, error) {
	tx, _, err := client.TransactionByHash(context.Background(), hash)
	if err != nil {
		return "", err
	}

	from, err := types.Sender(types.NewEIP155Signer(tx.ChainId()), tx)
	if err != nil {
		return "", err
	}

	msg := ethereum.CallMsg{
		From:     from,
		To:       tx.To(),
		Gas:      tx.Gas(),
		GasPrice: tx.GasPrice(),
		Value:    tx.Value(),
		Data:     tx.Data(),
	}

	wr.log.WithFields(logrus.Fields{
		"From":     from,
		"To":       tx.To(),
		"Gas":      tx.Gas(),
		"GasPrice": tx.GasPrice(),
		"Value":    tx.Value(),
		"Data":     hex.EncodeToString(tx.Data()),
	}).Info("Call info")

	// The logger does a test call to the actual contract to check for any revert message and log it, as well
	// as logging the call info. This is because the golang client can sometimes supress the log message and so
	// it can be helpful to use the call info to do the same call in Truffle/Web3js to get better logs.
	res, err := client.CallContract(context.Background(), msg, nil)
	if err != nil {
		return "", err
	}

	return string(res), nil
}
