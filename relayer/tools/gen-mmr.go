package main

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"fmt"
	"hash"

	"reflect"

	"github.com/cbergoon/merkletree"
	secp256k1 "github.com/ethereum/go-ethereum/crypto"
	"golang.org/x/crypto/blake2b"
	"golang.org/x/crypto/sha3"
)

// Validator keypair (secp256k1)
type Keypair struct {
	publicBytes []byte
	public      *ecdsa.PublicKey
	private     *ecdsa.PrivateKey
}

type ValidatorPublicKey []byte
type ParaBlockHash [32]byte

// Uninterpreted hash type
type H256 [32]byte

// Leaf node for MMR
type MMRLeaf struct {
	blockHash        H256
	paraHeadsRoot    H256
	hasValidatorRoot bool
	validatorRoot    H256
}

// hashing strategy for Merkle trees
func hashStrategy() hash.Hash {
	return sha3.NewLegacyKeccak256()
}

// Merkle Tree for ParaHeads
func (head ParaBlockHash) CalculateHash() ([]byte, error) {
	h := sha3.NewLegacyKeccak256()
	if _, err := h.Write(head[:]); err != nil {
		return nil, err
	}
	return h.Sum(nil), nil
}

func (head ParaBlockHash) Equals(other merkletree.Content) (bool, error) {
	return head == other, nil
}

// Merkle Tree for validator keys
func (key ValidatorPublicKey) CalculateHash() ([]byte, error) {
	h := sha3.NewLegacyKeccak256()
	if _, err := h.Write(key); err != nil {
		return nil, err
	}
	return h.Sum(nil), nil
}

func (key ValidatorPublicKey) Equals(other merkletree.Content) (bool, error) {
	return reflect.DeepEqual(key, other), nil
}

// Merkle Tree for MMRLeaf
func (leaf MMRLeaf) CalculateHash() ([]byte, error) {
	h := sha3.NewLegacyKeccak256()
	if _, err := h.Write(leaf.blockHash[:]); err != nil {
		return nil, err
	}
	return h.Sum(nil), nil
}

func (leaf MMRLeaf) Equals(other merkletree.Content) (bool, error) {
	return leaf.blockHash == other.(MMRLeaf).blockHash, nil
}

// Generate a merkle root hash for arbitrary leaf nodes
func merkleRoot(keys ...merkletree.Content) H256 {
	tree, err := merkletree.NewTreeWithHashStrategy(keys, hashStrategy)
	if err != nil {
		panic(err)
	}

	var hash [32]byte
	copy(hash[:], tree.MerkleRoot())
	return hash
}

// Create a random validator keypair (secp256k1)
func makeValidatorKeypair() *Keypair {
	pk, err := secp256k1.GenerateKey()
	if err != nil {
		panic(err)
	}

	public := pk.Public().(*ecdsa.PublicKey)

	return &Keypair{
		publicBytes: elliptic.Marshal(elliptic.P256(), public.X, public.Y),
		public:      public,
		private:     pk,
	}
}

// create a BLAKE2b hash for a mock relay chain header
func makeRelayChainHead(id byte) [32]byte {
	return blake2b.Sum256([]byte{id})
}

// create a KECCAK256 hash for a mock parachain header
func makeParaChainHead(id byte) ParaBlockHash {
	h := sha3.NewLegacyKeccak256()
	if _, err := h.Write([]byte{id}); err != nil {
		panic(err)
	}
	var head [32]byte
	copy(head[:], h.Sum(nil))
	return head
}

func main() {

	alice := makeValidatorKeypair()
	bob := makeValidatorKeypair()
	alicePubKey := ValidatorPublicKey(alice.publicBytes)
	bobPubKey := ValidatorPublicKey(bob.publicBytes)

	node0 := MMRLeaf{
		blockHash:        makeRelayChainHead(1),
		paraHeadsRoot:    merkleRoot(makeParaChainHead(1), makeParaChainHead(2)),
		hasValidatorRoot: true,
		validatorRoot:    merkleRoot(alicePubKey, bobPubKey),
	}

	node1 := MMRLeaf{
		blockHash:        makeRelayChainHead(2),
		hasValidatorRoot: false,
		paraHeadsRoot:    merkleRoot(makeParaChainHead(3), makeParaChainHead(4)),
	}

	node2 := MMRLeaf{
		blockHash:        makeRelayChainHead(3),
		hasValidatorRoot: false,
		paraHeadsRoot:    merkleRoot(makeParaChainHead(5), makeParaChainHead(6)),
	}

	node3 := MMRLeaf{
		blockHash:        makeRelayChainHead(4),
		hasValidatorRoot: false,
		paraHeadsRoot:    merkleRoot(makeParaChainHead(7), makeParaChainHead(8)),
	}

	tree, err := merkletree.NewTreeWithHashStrategy([]merkletree.Content{node0, node1, node2, node3}, hashStrategy)
	if err != nil {
		panic(err)
	}

	valid, err := tree.VerifyTree()
	if err != nil {
		panic(err)
	}

	fmt.Println("MMR Peak: ", tree.MerkleRoot())
	fmt.Println("MMR is valid: ", valid)

}
