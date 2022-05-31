package bitfield

import (
	"math/big"
)

type Bitfield []byte

func reverse(thing []byte) {
	for i, j := 0, len(thing)-1; i < j; i, j = i+1, j-1 {
		thing[i], thing[j] = thing[j], thing[i]
	}
}

/// New returns a Bitfield initialized from the Solidity representation of a Bitfield (an array of big integers). See below:
/// https://github.com/Snowfork/snowbridge/blob/18c6225b21782170156729d54a35404d876a2c7b/ethereum/contracts/utils/Bitfield.sol
func New(input []*big.Int) Bitfield {
	result := make(Bitfield, 256 * len(input))
	for i, chunk := range input {
		k := i * 256
		j := (i + 1) * 256
		chunk.FillBytes(result[k:j])
		reverse(result[k:j])
	}
	return result
}

/// Members returns the set bits in the bitfield
func (b Bitfield) Members() []uint64 {
	results := []uint64{}
	for idx, bits := range b {
		if bits == 0 {
			continue
		}
		for i := 0; i < 8; i++ {
			if bits & byte(1 << i) > 0 {
				results = append(results, uint64(idx + i))
			}
		}
	}
	return results
}
