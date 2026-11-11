package state

// Gloas progressive HashTreeRoot / Merkle proofs go through dynamic-ssz
// (Apache-2.0), which implements EIP-7495/7916. fastssz still has no progressive
// support and would silently emit classic roots for these types.

import dynssz "github.com/pk910/dynamic-ssz"

// gloasSSZ is shared for Gloas body/bid/state hashing and proofs. Spec map is nil:
// mainnet/minimal sizes are carried on struct tags (ssz-size / ssz-max).
var gloasSSZ = dynssz.NewDynSsz(nil)
