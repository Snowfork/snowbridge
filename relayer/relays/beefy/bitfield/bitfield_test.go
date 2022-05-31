package bitfield

import (
	"fmt"
	"math/big"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestFoo(t *testing.T) {
	x := big.NewInt(1)
	y := big.NewInt(1)
	z := big.NewInt(1)

	u := New([]*big.Int{x, y, z})
	fmt.Printf("%v\n", u)
	fmt.Printf("%v\n", u.Members())

	assert.Equal(t, u.Members(), []uint64{0, 256, 512})
}
