package bitfield

import (
	"fmt"
	"math/big"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestBitfieldMembers(t *testing.T) {
	x := big.NewInt(1)
	y := big.NewInt(1)
	z := big.NewInt(1)

	u := New([]*big.Int{x, y, z})
	fmt.Printf("%v\n", u)
	fmt.Printf("%v\n", u.Members())

	assert.Equal(t, u.Members(), []uint64{0, 256, 512})
}

func TestBitfieldMembers2(t *testing.T) {
	foo := make([]byte, 256)
	foo[0] = 1
	foo[255] = 1

	x := big.NewInt(1)
	x.SetBytes(foo)

	u := New([]*big.Int{x})
	fmt.Printf("%v\n", u)
	fmt.Printf("%v\n", u.Members())

	assert.Equal(t, u.Members(), []uint64{0, 255,})
}
