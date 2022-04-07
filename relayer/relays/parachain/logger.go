package parachain

import (
	log "github.com/sirupsen/logrus"
	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/contracts/basic"
	"github.com/snowfork/snowbridge/relayer/contracts/incentivized"
	"github.com/snowfork/snowbridge/relayer/crypto/keccak"
)

func Hex(b []byte) string {
	return types.HexEncodeToString(b)
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

	var messagesLog []log.Fields
	for _, item := range messages {
		messagesLog = append(messagesLog, log.Fields{
			"target":  item.Target,
			"nonce":   item.Nonce,
			"payload": Hex(item.Payload),
		})
	}
	var paraHeadProofString []string
	for _, item := range paraVerifyInput.ParachainHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, Hex(item[:]))
	}

	var beefyMMRMerkleProofItems []string
	for _, item := range beefyMMRSimplifiedProof.MerkleProofItems {
		beefyMMRMerkleProofItems = append(beefyMMRMerkleProofItems, Hex(item[:]))
	}

	submit := log.Fields{
		"messages": messagesLog,
		"paraVerifyInput": log.Fields{
			"ownParachainHeadPrefixBytes": Hex(paraVerifyInput.OwnParachainHeadPrefixBytes),
			"ownParachainHeadSuffixBytes": Hex(paraVerifyInput.OwnParachainHeadSuffixBytes),
			"parachainHeadProof": log.Fields{
				"pos":   paraVerifyInput.ParachainHeadProof.Pos,
				"width": paraVerifyInput.ParachainHeadProof.Width,
				"proof": paraHeadProofString,
			},
		},
		"leafPartial": log.Fields{
			"version":              beefyMMRLeafPartial.Version,
			"parentNumber":         beefyMMRLeafPartial.ParentNumber,
			"parentHash":           Hex(beefyMMRLeafPartial.ParentHash[:]),
			"nextAuthoritySetId":   beefyMMRLeafPartial.NextAuthoritySetId,
			"nextAuthoritySetLen":  beefyMMRLeafPartial.NextAuthoritySetLen,
			"nextAuthoritySetRoot": Hex(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		},
		"proof": log.Fields{
			"merkleProofItems": beefyMMRMerkleProofItems,
			"merkleProofOrder": beefyMMRSimplifiedProof.MerkleProofOrderBitField,
		},
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

	hasher := &keccak.Keccak256{}

	log.WithFields(log.Fields{
		"submit":                      submit,
		"commitmentHash":              Hex(commitmentHash[:]),
		"paraHeadProofRootMerkleLeaf": Hex(mmrLeaf.ParachainHeads[:]),
		"mmrLeafOpaqueEncoded":        mmrLeafOpaqueEncoded,
		"hashedOpaqueLeaf":            Hex(hasher.Hash(mmrLeafOpaqueEncodedBytes)),
		"hashedLeaf":                  Hex(hasher.Hash(mmrLeafEncoded)),
		"mmrRootHash":                 Hex(mmrRootHash[:]),
		"merkleProofData":             merkleProofData,
		"scaleParaId":                 scaleParaID,
		"scaleParaHead":               scaleParaHead,
		"scaleParaHeadParentHash":     scaleParaHeadParentHash,
		"scaleparaHeadNumber":         scaleparaHeadNumber,
		"scaleparaHeadStateRoot":      scaleparaHeadStateRoot,
		"scaleparaHeadExtrinsicsRoot": scaleparaHeadExtrinsicsRoot,
		"scaleparaHeadDigest":         scaleparaHeadDigest,
		"scaleDigestItems":            scaleDigestItems,
	}).Info("Message submission to basic channel")

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
	var messagesLog []log.Fields
	for _, item := range messages {
		messagesLog = append(messagesLog, log.Fields{
			"target":  item.Target,
			"nonce":   item.Nonce,
			"fee":     item.Fee,
			"payload": Hex(item.Payload),
		})
	}

	var paraHeadProofString []string
	for _, item := range paraVerifyInput.ParachainHeadProof.Proof {
		paraHeadProofString = append(paraHeadProofString, Hex(item[:]))
	}

	var beefyMMRMerkleProofItems []string
	for _, item := range beefyMMRSimplifiedProof.MerkleProofItems {
		beefyMMRMerkleProofItems = append(beefyMMRMerkleProofItems, Hex(item[:]))
	}

	submit := log.Fields{
		"messages": messagesLog,
		"paraVerifyInput": log.Fields{
			"ownParachainHeadPrefixBytes": Hex(paraVerifyInput.OwnParachainHeadPrefixBytes),
			"ownParachainHeadSuffixBytes": Hex(paraVerifyInput.OwnParachainHeadSuffixBytes),
			"parachainHeadProof": log.Fields{
				"pos":   paraVerifyInput.ParachainHeadProof.Pos,
				"width": paraVerifyInput.ParachainHeadProof.Width,
				"proof": paraHeadProofString,
			},
		},
		"leafPartial": log.Fields{
			"version":              beefyMMRLeafPartial.Version,
			"parentNumber":         beefyMMRLeafPartial.ParentNumber,
			"parentHash":           Hex(beefyMMRLeafPartial.ParentHash[:]),
			"nextAuthoritySetId":   beefyMMRLeafPartial.NextAuthoritySetId,
			"nextAuthoritySetLen":  beefyMMRLeafPartial.NextAuthoritySetLen,
			"nextAuthoritySetRoot": Hex(beefyMMRLeafPartial.NextAuthoritySetRoot[:]),
		},
		"proof": log.Fields{
			"merkleProofItems": beefyMMRMerkleProofItems,
			"merkleProofOrder": beefyMMRSimplifiedProof.MerkleProofOrderBitField,
		},
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

	hasher := &keccak.Keccak256{}

	log.WithFields(log.Fields{
		"submit":                      submit,
		"commitmentHash":              Hex(commitmentHash[:]),
		"paraHeadProofRootMerkleLeaf": Hex(mmrLeaf.ParachainHeads[:]),
		"mmrLeafOpaqueEncoded":        mmrLeafOpaqueEncoded,
		"mmrRootHash":                 Hex(mmrRootHash[:]),
		"merkleProofData":             merkleProofData,
		"hashedOpaqueLeaf":            Hex(hasher.Hash(mmrLeafOpaqueEncodedBytes)),
		"hashedLeaf":                  Hex(hasher.Hash(mmrLeafEncoded)),
		"scaleParaId":                 scaleParaID,
		"scaleParaHead":               scaleParaHead,
		"scaleParaHeadParentHash":     scaleParaHeadParentHash,
		"scaleparaHeadNumber":         scaleparaHeadNumber,
		"scaleparaHeadStateRoot":      scaleparaHeadStateRoot,
		"scaleparaHeadExtrinsicsRoot": scaleparaHeadExtrinsicsRoot,
		"scaleparaHeadDigest":         scaleparaHeadDigest,
		"scaleDigestItems":            scaleDigestItems,
	}).Info("Message submission to incentivized channel")
	return nil
}
