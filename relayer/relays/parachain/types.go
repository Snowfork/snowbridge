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
	HeaderProof      [][32]byte
	HeaderProofPos   int
	HeaderProofWidth int
	HeaderProofRoot  []byte
}

type DigestItemWithData struct {
	DigestItem parachain.AuxiliaryDigestItem
	Data       types.StorageDataRaw
}

type MessagePackage struct {
	channelID          parachain.ChannelID
	commitmentHash     types.H256
	commitmentData     types.StorageDataRaw
	paraHead           types.Header
	paraHeadProof      [][32]byte
	paraHeadProofPos   int
	paraHeadProofWidth int
	paraHeadProofRoot  []byte
	mmrProof           types.GenerateMMRProofResponse
	mmrProofLeafCount  uint64
}

func CreateMessagePackages(paraBlocks []ParaBlockWithProofs, relaychainBlock uint64) ([]MessagePackage, error) {
	var messagePackages []MessagePackage

	// This leaf count calculation assumes that there is one mmr leaf for every relay chain block
	// except the newest, with no pruning.
	// TODO: verify the assumption/remove it if possible
	mmrProofLeafCount := relaychainBlock - 1

	for _, block := range paraBlocks {
		for _, item := range block.Block.DigestItemsWithData {
			commitmentHash := item.DigestItem.AsCommitment.Hash
			commitmentData := item.Data
			messagePackage := MessagePackage{
				item.DigestItem.AsCommitment.ChannelID,
				commitmentHash,
				commitmentData,
				block.Header,
				block.HeaderProof,
				block.HeaderProofPos,
				block.HeaderProofWidth,
				block.HeaderProofRoot,
				block.MMRProofResponse,
				mmrProofLeafCount,
			}
			messagePackages = append(messagePackages, messagePackage)
		}
	}

	return messagePackages, nil
}
