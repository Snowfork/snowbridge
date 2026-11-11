package beaconstate

import (
	"fmt"

	ssz "github.com/ferranbt/fastssz"
	"github.com/pk910/dynamic-ssz/treeproof"
	"github.com/snowfork/snowbridge/relayer/relays/beacon/state"
)

// prover produces a merkle proof for a generalized index. Pre-Gloas states use fastssz's
// tree; Gloas states use dynamic-ssz's progressive tree.
type prover interface {
	prove(gindex int) (leaf []byte, hashes [][]byte, err error)
}

// fastsszProver proves against a classic fastssz tree.
type fastsszProver struct{ tree *ssz.Node }

func (p fastsszProver) prove(gindex int) ([]byte, [][]byte, error) {
	proof, err := p.tree.Prove(gindex)
	if err != nil {
		return nil, nil, err
	}
	return proof.Leaf, proof.Hashes, nil
}

// dynsszProver proves against a progressive tree built by dynamic-ssz.
type dynsszProver struct{ tree *treeproof.Node }

func (p dynsszProver) prove(gindex int) ([]byte, [][]byte, error) {
	proof, err := p.tree.Prove(gindex)
	if err != nil {
		return nil, nil, err
	}
	return proof.Leaf, proof.Hashes, nil
}

// newProver returns the prover appropriate to the state's fork.
func newProver(beaconState state.BeaconState) (prover, error) {
	if gloas, ok := beaconState.(*state.BeaconStateGloas); ok {
		tree, err := gloas.GloasProofTree()
		if err != nil {
			return nil, fmt.Errorf("gloas proof tree: %w", err)
		}
		return dynsszProver{tree: tree}, nil
	}
	tree, err := beaconState.GetTree()
	if err != nil {
		return nil, err
	}
	return fastsszProver{tree: tree}, nil
}
