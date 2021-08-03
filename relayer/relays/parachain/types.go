package parachain

import (
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/snowbridge/relayer/chain/parachain"
)

type ParaBlockWithDigest struct {
	BlockNumber         uint64
	DigestItemsWithData []DigestItemWithData
}

type ParaBlockWithProofs struct {
	Block            ParaBlockWithDigest
	MMRProofResponse types.GenerateMMRProofResponse
	Header           types.Header
	MerkleProofData  MerkleProofData
}

type DigestItemWithData struct {
	DigestItem parachain.AuxiliaryDigestItem
	Data       types.StorageDataRaw
}

type MessagePackage struct {
	channelID         parachain.ChannelID
	commitmentHash    types.H256
	commitmentData    types.StorageDataRaw
	paraHead          types.Header
	merkleProofData   MerkleProofData
	mmrProof          types.GenerateMMRProofResponse
	mmrProofLeafCount uint64
}

func CreateMessagePackages(paraBlocks []ParaBlockWithProofs, mmrLeafCount uint64) ([]MessagePackage, error) {
	var messagePackages []MessagePackage

	for _, block := range paraBlocks {
		for _, item := range block.Block.DigestItemsWithData {
			commitmentHash := item.DigestItem.AsCommitment.Hash
			commitmentData := item.Data
			messagePackage := MessagePackage{
				item.DigestItem.AsCommitment.ChannelID,
				commitmentHash,
				commitmentData,
				block.Header,
				block.MerkleProofData,
				block.MMRProofResponse,
				mmrLeafCount,
			}
			messagePackages = append(messagePackages, messagePackage)
		}
	}

	return messagePackages, nil
}
