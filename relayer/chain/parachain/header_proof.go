package parachain

import (
	"fmt"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/wealdtech/go-merkletree"
)

func CreateParachainHeaderProof(allParaHeads []types.Bytes, ourParaHead types.Header, expectedRoot types.H256) (
	[]byte, [][32]byte, error) {
	ourParaHeadBytes, err := types.EncodeToBytes(ourParaHead)
	if err != nil {
		return nil, [][32]byte{}, err
	}

	paraTreeData := make([][]byte, len(allParaHeads))
	for i, paraHead := range allParaHeads {
		paraTreeData[i] = paraHead
	}

	// Create the tree
	paraMerkleTree, err := merkletree.NewUsing(paraTreeData, &Keccak256{}, nil)
	if err != nil {
		return nil, [][32]byte{}, err
	}

	// Generate Merkle Proof for our parachain's head
	proof, err := paraMerkleTree.GenerateProof(ourParaHeadBytes)
	if err != nil {
		return nil, [][32]byte{}, err
	}
	root := paraMerkleTree.Root()

	// Check the root matches our expected root  - TODO, uncomment this once beefy is fixed
	// for i, v := range expectedRoot[0:32] {
	// 	if v != root[i] {
	// 		return [][32]byte{}, fmt.Errorf("Merkle tree created does not match the expected root")
	// 	}
	// }

	// Verify the proof
	verified, err := merkletree.VerifyProofUsing(ourParaHeadBytes, proof, root, &Keccak256{}, nil)
	if err != nil {
		return nil, [][32]byte{}, err
	}
	if !verified {
		return nil, [][32]byte{}, fmt.Errorf("failed to verify proof")
	}

	proofContents := make([][32]byte, len(proof.Hashes))
	for i, hash := range proof.Hashes {
		var hash32Byte [32]byte
		copy(hash32Byte[:], hash)
		proofContents[i] = hash32Byte
	}

	return root, proofContents, nil
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
