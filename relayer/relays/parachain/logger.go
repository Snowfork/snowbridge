package parachain

import (
	"encoding/hex"
	"encoding/json"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
)

type ParaVerifyInputLog struct {
	OwnParachainHeadPrefixBytes string           `json:"ownParachainHeadPrefixBytes"`
	OwnParachainHeadSuffixBytes string           `json:"ownParachainHeadSuffixBytes"`
	ParachainHeadProof          ParaHeadProofLog `json:"parachainHeadProof"`
}

type ParaHeadProofLog struct {
	Pos   *big.Int `json:"pos"`
	Width *big.Int `json:"width"`
	Proof []string `json:"proof"`
}

type BeefyMMRLeafPartialLog struct {
	Version              uint8  `json:"version"`
	ParentNumber         uint32 `json:"parentNumber"`
	ParentHash           string `json:"parentHash"`
	NextAuthoritySetId   uint64 `json:"nextAuthoritySetId"` // revive:disable-line
	NextAuthoritySetLen  uint32 `json:"nextAuthoritySetLen"`
	NextAuthoritySetRoot string `json:"nextAuthoritySetRoot"`
}

type BasicInboundChannelMessageLog struct {
	Target  common.Address `json:"target"`
	Nonce   uint64         `json:"nonce"`
	Payload string         `json:"payload"`
}

type IncentivizedInboundChannelMessageLog struct {
	Target  common.Address `json:"target"`
	Nonce   uint64         `json:"nonce"`
	Fee     *big.Int       `json:"fee"`
	Payload string         `json:"payload"`
}

type SimplifiedMMRProofLog struct {
	BeefyMMRRestOfThePeaks  []string `json:"RestOfThePeaks"`
	BeefyMMRRightBaggedPeak string   `json:"RightBaggedPeak"`
	MerkleProofItems        []string `json:"MerkleProofItems"`
	MerkleProofOrder        uint64   `json:"MerkleProofOrder"`
}

type BasicSubmitInput struct {
	Messages            []BasicInboundChannelMessageLog `json:"_messages"`
	ParaVerifyInput     ParaVerifyInputLog              `json:"_paraVerifyInput"`
	BeefyMMRLeafPartial BeefyMMRLeafPartialLog          `json:"_beefyMMRLeafPartial"`
	SimplifiedMMRProof  SimplifiedMMRProofLog           `json:"_beefyMMRSimplifiedProof"`
}

type IncentivizedSubmitInput struct {
	Messages            []IncentivizedInboundChannelMessageLog `json:"_messages"`
	ParaVerifyInput     ParaVerifyInputLog                     `json:"_paraVerifyInput"`
	BeefyMMRLeafPartial BeefyMMRLeafPartialLog                 `json:"_beefyMMRLeafPartial"`
	SimplifiedMMRProof  SimplifiedMMRProofLog                  `json:"_beefyMMRSimplifiedProof"`
}

func (wr *EthereumChannelWriter) logBasicTx(
	messages []basic.BasicInboundChannelMessage,
	paraVerifyInput basic.ParachainLightClientParachainVerifyInput,
	beefyMMRLeafPartial basic.ParachainLightClientBeefyMMRLeafPartial,
	beefyMMRSimplifiedProof basic.SimplifiedMMRProof,
	paraHead *types.Header,
	merkleProofData MerkleProofData,
	mmrLeaf types.MMRLeaf,
	commitmentHash types.H256,
	paraID uint32,
	mmrRootHash types.Hash,
) error {

	var basicMessagesLog []BasicInboundChannelMessageLog
	for _, item := range messages {
		basicMessagesLog = append(basicMessagesLog, BasicInboundChannelMessageLog{
			Target:  item.Target,
			Nonce:   item.Nonce,
			Payload: "0x" + hex.EncodeToString(item.Payload),
		})
	}
	var paraHeadProofString []string
	for _, item := range paraVerifyInput.ParachainHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, "0x"+hex.EncodeToString(item[:]))
	}

	var beefyMMRMerkleProofItems []string
	for _, item := range beefyMMRSimplifiedProof.MerkleProofItems {
		beefyMMRMerkleProofItems = append(beefyMMRMerkleProofItems, "0x"+hex.EncodeToString(item[:]))
	}

	input := &BasicSubmitInput{
		Messages: basicMessagesLog,
		ParaVerifyInput: ParaVerifyInputLog{
			OwnParachainHeadPrefixBytes: "0x" + hex.EncodeToString(paraVerifyInput.OwnParachainHeadPrefixBytes),
			OwnParachainHeadSuffixBytes: "0x" + hex.EncodeToString(paraVerifyInput.OwnParachainHeadSuffixBytes),
			ParachainHeadProof: ParaHeadProofLog{
				Pos:   paraVerifyInput.ParachainHeadProof.Pos,
				Width: paraVerifyInput.ParachainHeadProof.Width,
				Proof: paraHeadProofString,
			},
		},
		BeefyMMRLeafPartial: BeefyMMRLeafPartialLog{
			Version:              beefyMMRLeafPartial.Version,
			ParentNumber:         beefyMMRLeafPartial.ParentNumber,
			ParentHash:           "0x" + hex.EncodeToString(beefyMMRLeafPartial.ParentHash[:]),
			NextAuthoritySetId:   beefyMMRLeafPartial.NextAuthoritySetId,
			NextAuthoritySetLen:  beefyMMRLeafPartial.NextAuthoritySetLen,
			NextAuthoritySetRoot: "0x" + hex.EncodeToString(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		},
		SimplifiedMMRProof: SimplifiedMMRProofLog{
			MerkleProofItems: beefyMMRMerkleProofItems,
			MerkleProofOrder: beefyMMRSimplifiedProof.MerkleProofOrderBitField,
		},
	}
	b, err := json.Marshal(input)
	if err != nil {
		return err
	}

	mmrLeafEncoded, _ := types.EncodeToBytes(mmrLeaf)
	mmrLeafOpaqueEncoded, _ := types.EncodeToHexString(mmrLeafEncoded)
	mmrLeafOpaqueEncodedBytes, _ := types.EncodeToBytes(mmrLeafEncoded)
	scaleParaID, _ := types.EncodeToHexString(paraID)
	scaleParaHead, _ := types.EncodeToHexString(paraHead)
	scaleParaHeadParentHash, _ := types.EncodeToHexString(paraHead.ParentHash)
	scaleparaHeadNumber, _ := types.EncodeToHexString(paraHead.Number)
	scaleparaHeadStateRoot, _ := types.EncodeToHexString(paraHead.StateRoot)
	scaleparaHeadExtrinsicsRoot, _ := types.EncodeToHexString(paraHead.ExtrinsicsRoot)
	scaleparaHeadDigest, _ := types.EncodeToHexString(paraHead.Digest)
	var scaleDigestItems []string
	for _, item := range paraHead.Digest {
		scaleDigestItem, _ := types.EncodeToHexString(item)
		scaleDigestItems = append(scaleDigestItems, scaleDigestItem)
	}

	log.WithFields(log.Fields{
		"input":                       string(b),
		"commitmentHash":              "0x" + hex.EncodeToString(commitmentHash[:]),
		"paraHeadProofRootMerkleLeaf": "0x" + hex.EncodeToString(mmrLeaf.ParachainHeads[:]),
		"mmrLeafOpaqueEncoded":        mmrLeafOpaqueEncoded,
		"mmrRootHash":                 "0x" + hex.EncodeToString(mmrRootHash[:]),
		"merkleProofData":             merkleProofData,
		"scaleParaId":                 scaleParaID,
		"scaleParaHead":               scaleParaHead,
		"scaleParaHeadParentHash":     scaleParaHeadParentHash,
		"scaleparaHeadNumber":         scaleparaHeadNumber,
		"scaleparaHeadStateRoot":      scaleparaHeadStateRoot,
		"scaleparaHeadExtrinsicsRoot": scaleparaHeadExtrinsicsRoot,
		"scaleparaHeadDigest":         scaleparaHeadDigest,
		"scaleDigestItems":            scaleDigestItems,
	}).Info("Submitting tx")

	hasher := &keccak.Keccak256{}

	log.WithFields(log.Fields{
		"mmrLeafOpaqueEncoded": mmrLeafOpaqueEncoded,
		"hashedOpaqueLeaf":     "0x" + hex.EncodeToString(hasher.Hash(mmrLeafOpaqueEncodedBytes)),
		"hashedLeaf":           "0x" + hex.EncodeToString(hasher.Hash(mmrLeafEncoded)),
	}).Info("DAT LEAF")

	return nil
}

func (wr *EthereumChannelWriter) logIncentivizedTx(
	messages []incentivized.IncentivizedInboundChannelMessage,
	paraVerifyInput incentivized.ParachainLightClientParachainVerifyInput,
	beefyMMRLeafPartial incentivized.ParachainLightClientBeefyMMRLeafPartial,
	beefyMMRSimplifiedProof incentivized.SimplifiedMMRProof,
	paraHead *types.Header,
	merkleProofData MerkleProofData,
	mmrLeaf types.MMRLeaf,
	commitmentHash types.H256,
	paraID uint32,
	mmrRootHash types.Hash,
) error {
	var incentivizedMessagesLog []IncentivizedInboundChannelMessageLog
	for _, item := range messages {
		incentivizedMessagesLog = append(incentivizedMessagesLog, IncentivizedInboundChannelMessageLog{
			Target:  item.Target,
			Nonce:   item.Nonce,
			Fee:     item.Fee,
			Payload: "0x" + hex.EncodeToString(item.Payload),
		})
	}

	var paraHeadProofString []string
	for _, item := range paraVerifyInput.ParachainHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, "0x"+hex.EncodeToString(item[:]))
	}

	var beefyMMRMerkleProofItems []string
	for _, item := range beefyMMRSimplifiedProof.MerkleProofItems {
		beefyMMRMerkleProofItems = append(beefyMMRMerkleProofItems, "0x"+hex.EncodeToString(item[:]))
	}

	input := &IncentivizedSubmitInput{
		Messages: incentivizedMessagesLog,
		ParaVerifyInput: ParaVerifyInputLog{
			OwnParachainHeadPrefixBytes: "0x" + hex.EncodeToString(paraVerifyInput.OwnParachainHeadPrefixBytes),
			OwnParachainHeadSuffixBytes: "0x" + hex.EncodeToString(paraVerifyInput.OwnParachainHeadSuffixBytes),
			ParachainHeadProof: ParaHeadProofLog{
				Pos:   paraVerifyInput.ParachainHeadProof.Pos,
				Width: paraVerifyInput.ParachainHeadProof.Width,
				Proof: paraHeadProofString,
			},
		},
		BeefyMMRLeafPartial: BeefyMMRLeafPartialLog{
			ParentNumber:         beefyMMRLeafPartial.ParentNumber,
			ParentHash:           "0x" + hex.EncodeToString(beefyMMRLeafPartial.ParentHash[:]),
			NextAuthoritySetId:   beefyMMRLeafPartial.NextAuthoritySetId,
			NextAuthoritySetLen:  beefyMMRLeafPartial.NextAuthoritySetLen,
			NextAuthoritySetRoot: "0x" + hex.EncodeToString(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		},
		SimplifiedMMRProof: SimplifiedMMRProofLog{
			MerkleProofItems: beefyMMRMerkleProofItems,
			MerkleProofOrder: beefyMMRSimplifiedProof.MerkleProofOrderBitField,
		},
	}
	b, err := json.Marshal(input)
	if err != nil {
		return err
	}

	mmrLeafEncoded, _ := types.EncodeToBytes(mmrLeaf)
	mmrLeafOpaqueEncoded, _ := types.EncodeToHexString(mmrLeafEncoded)
	mmrLeafOpaqueEncodedBytes, _ := types.EncodeToBytes(mmrLeafEncoded)
	scaleParaID, _ := types.EncodeToHexString(paraID)
	scaleParaHead, _ := types.EncodeToHexString(paraHead)
	scaleParaHeadParentHash, _ := types.EncodeToHexString(paraHead.ParentHash)
	scaleparaHeadNumber, _ := types.EncodeToHexString(paraHead.Number)
	scaleparaHeadStateRoot, _ := types.EncodeToHexString(paraHead.StateRoot)
	scaleparaHeadExtrinsicsRoot, _ := types.EncodeToHexString(paraHead.ExtrinsicsRoot)
	scaleparaHeadDigest, _ := types.EncodeToHexString(paraHead.Digest)
	var scaleDigestItems []string
	for _, item := range paraHead.Digest {
		scaleDigestItem, _ := types.EncodeToHexString(item)
		scaleDigestItems = append(scaleDigestItems, scaleDigestItem)
	}

	log.WithFields(log.Fields{
		"input":                       string(b),
		"commitmentHash":              "0x" + hex.EncodeToString(commitmentHash[:]),
		"paraHeadProofRootMerkleLeaf": "0x" + hex.EncodeToString(mmrLeaf.ParachainHeads[:]),
		"mmrLeafOpaqueEncoded":        mmrLeafOpaqueEncoded,
		"mmrRootHash":                 "0x" + hex.EncodeToString(mmrRootHash[:]),
		"merkleProofData":             merkleProofData,
		"scaleParaId":                 scaleParaID,
		"scaleParaHead":               scaleParaHead,
		"scaleParaHeadParentHash":     scaleParaHeadParentHash,
		"scaleparaHeadNumber":         scaleparaHeadNumber,
		"scaleparaHeadStateRoot":      scaleparaHeadStateRoot,
		"scaleparaHeadExtrinsicsRoot": scaleparaHeadExtrinsicsRoot,
		"scaleparaHeadDigest":         scaleparaHeadDigest,
		"scaleDigestItems":            scaleDigestItems,
	}).Info("Submitting tx")

	hasher := &keccak.Keccak256{}

	log.WithFields(log.Fields{
		"mmrLeafOpaqueEncoded": mmrLeafOpaqueEncoded,
		"hashedOpaqueLeaf":     "0x" + hex.EncodeToString(hasher.Hash(mmrLeafOpaqueEncodedBytes)),
		"hashedLeaf":           "0x" + hex.EncodeToString(hasher.Hash(mmrLeafEncoded)),
	}).Info("DAT LEAF")
	return nil
}
