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

func (b Bitfield) Members() []uint64 {
	results := []uint64{}
	for idx, bits := range b {
		for i := 0; i < 8; i++ {
			if bits & byte(1 << i) > 0 {
				results = append(results, uint64(idx + i))
			}
		}
	}
	return results
}
