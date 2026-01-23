package merkle

import (
	"bytes"
	"errors"
	"fmt"
	"hash"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"golang.org/x/crypto/sha3"
)

// Content represents the data that is stored and verified by the tree. A type that
// implements this interface can be used as an item in the tree.
type Content interface {
	CalculateHash() ([]byte, error)
	Equals(other Content) (bool, error)
}

// MerkleTree is the container for the tree. It holds a pointer to the root of the tree,
// a list of pointers to the leaf nodes, and the merkle root.
type MerkleTree struct {
	Root         *Node2
	merkleRoot   []byte
	Leafs        []*Node2
	hashStrategy func() hash.Hash
	sort         bool
}

// Node represents a node, root, or leaf in the tree. It stores pointers to its immediate
// relationships, a hash, the content stored if it is a leaf, and other metadata.
type Node2 struct {
	Tree   *MerkleTree
	Parent *Node2
	Left   *Node2
	Right  *Node2
	leaf   bool
	dup    bool
	Hash   []byte
	C      Content
	sort   bool
	single bool
}

// Keccak256Content implements the Content interface provided by merkletree and represents the content stored in the tree.
type Keccak256Content struct {
	X types.H256
}

// CalculateHash hashes the values of a Keccak256Content
func (t Keccak256Content) CalculateHash() ([]byte, error) {
	return t.X[:], nil
}

// Equals tests for equality of two Contents
func (t Keccak256Content) Equals(other Content) (bool, error) {
	return t.X == other.(Keccak256Content).X, nil
}

// CalculateNodeHash is a helper function that calculates the hash of the node.
func (n *Node2) CalculateNodeHash(sort bool) ([]byte, error) {
	if n.leaf {
		return n.C.CalculateHash()
	}

	h := n.Tree.hashStrategy()
	if _, err := h.Write(combineTwoHash(n.Left.Hash, n.Right.Hash)); err != nil {
		return nil, err
	}

	return h.Sum(nil), nil
}

// NewTree2 creates a new Merkle Tree using the content cs.
func NewTree2(cs []Content) (*MerkleTree, error) {
	return NewTreeWithHashStrategy(cs, sha3.NewLegacyKeccak256)
}

// NewTreeWithHashStrategy creates a new Merkle Tree using the content cs using the provided hash
// strategy. Note that the hash type used in the type that implements the Content interface must
// match the hash type provided to the tree.
func NewTreeWithHashStrategy(cs []Content, hashStrategy func() hash.Hash) (*MerkleTree, error) {
	t := &MerkleTree{
		hashStrategy: hashStrategy,
	}
	root, leafs, err := buildWithContent(cs, t)
	if err != nil {
		return nil, err
	}
	t.Root = root
	t.Leafs = leafs
	t.merkleRoot = root.Hash
	return t, nil
}

// MerklePath: Get Merkle path and indexes(left leaf or right leaf)
func (m *MerkleTree) MerklePath(content Content) ([][]byte, []int64, error) {
	for _, current := range m.Leafs {
		ok, err := current.C.Equals(content)
		if err != nil {
			return nil, nil, err
		}

		if ok {
			if len(m.Leafs) == 1 {
				// only one leaf
				return [][]byte{}, []int64{}, nil
			}
			currentParent := current.Parent
			var merklePath [][]byte
			var index []int64
			for currentParent != nil {
				if bytes.Equal(currentParent.Left.Hash, current.Hash) {
					merklePath = append(merklePath, currentParent.Right.Hash)
					index = append(index, 1) // right leaf
				} else {
					merklePath = append(merklePath, currentParent.Left.Hash)
					index = append(index, 0) // left leaf
				}
				current = currentParent
				currentParent = currentParent.Parent
			}
			return merklePath, index, nil
		}
	}
	return nil, nil, nil
}

// buildWithContent is a helper function that for a given set of Contents, generates a
// corresponding tree and returns the root node, a list of leaf nodes, and a possible error.
// Returns an error if cs contains no Contents.
func buildWithContent(cs []Content, t *MerkleTree) (*Node2, []*Node2, error) {
	if len(cs) == 0 {
		return nil, nil, errors.New("error: cannot construct tree with no content")
	}
	var leafs []*Node2
	for _, c := range cs {
		hash, err := c.CalculateHash()
		if err != nil {
			return nil, nil, err
		}

		leafs = append(leafs, &Node2{
			Hash: hash,
			C:    c,
			leaf: true,
			Tree: t,
		})
	}
	root, err := buildIntermediate(leafs, t)
	if err != nil {
		return nil, nil, err
	}

	return root, leafs, nil
}

// buildIntermediate is a helper function that for a given list of leaf nodes, constructs
// the intermediate and root levels of the tree. Returns the resulting root node of the tree.
func buildIntermediate(nl []*Node2, t *MerkleTree) (*Node2, error) {
	var nodes []*Node2
	for i := 0; i < len(nl); i += 2 {
		h := t.hashStrategy()
		var left, right int = i, i + 1
		if i+1 == len(nl) {
			right = i
		}
		var nextHash []byte
		if left != right {
			// appear in pairs
			// compare their child hashes when doing combine
			if _, err := h.Write(combineTwoHash(nl[left].Hash, nl[right].Hash)); err != nil {
				return nil, err
			}
			nextHash = h.Sum(nil)
		} else {
			// single node
			// don't compute new hash
			nextHash = nl[right].Hash
			nl[right].single = true
		}
		n := &Node2{
			Left:  nl[left],
			Right: nl[right],
			Hash:  nextHash,
			Tree:  t,
		}
		nodes = append(nodes, n)
		nl[left].Parent = n
		nl[right].Parent = n
		if len(nl) == 2 || len(nl) == 1 {
			return n, nil
		}
	}
	return buildIntermediate(nodes, t)
}

// MerkleRoot returns the unverified Merkle Root (hash of the root node) of the tree.
func (m *MerkleTree) MerkleRoot() []byte {
	return m.merkleRoot
}

// RebuildTree is a helper function that will rebuild the tree reusing only the content that
// it holds in the leaves.
func (m *MerkleTree) RebuildTree() error {
	var cs []Content
	for _, c := range m.Leafs {
		cs = append(cs, c.C)
	}
	root, leafs, err := buildWithContent(cs, m)
	if err != nil {
		return err
	}
	m.Root = root
	m.Leafs = leafs
	m.merkleRoot = root.Hash
	return nil
}

// RebuildTreeWith replaces the content of the tree and does a complete rebuild; while the root of
// the tree will be replaced the MerkleTree completely survives this operation. Returns an error if the
// list of content cs contains no entries.
func (m *MerkleTree) RebuildTreeWith(cs []Content) error {
	root, leafs, err := buildWithContent(cs, m)
	if err != nil {
		return err
	}
	m.Root = root
	m.Leafs = leafs
	m.merkleRoot = root.Hash
	return nil
}

// String returns a string representation of the node.
func (n *Node2) String() string {
	return fmt.Sprintf("%t %t %v %s", n.leaf, n.dup, n.Hash, n.C)
}

// String returns a string representation of the tree. Only leaf nodes are included
// in the output.
func (m *MerkleTree) String() string {
	s := ""
	for _, l := range m.Leafs {
		s += fmt.Sprintf("%v", l)
		s += "\n"
	}
	return s
}

// ----------------------------------------------------------------------------

func combineTwoHash(a, b []byte) []byte {
	bf := bytes.NewBuffer(nil)
	if bytes.Compare(a, b) < 0 {
		bf.Write(a)
		bf.Write(b)
		return bf.Bytes()
	}

	bf.Write(b)
	bf.Write(a)
	return bf.Bytes()
}
