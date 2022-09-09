package parachain

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGenerateHashSidesSingleLeaf(t *testing.T) {
	sides, err := generateHashSides(0, 1)
	if err != nil {
		panic(err)
	}

	assert.Equal(t,
		[]bool{},
		sides,
	)
}

func TestGenerateHashSidesProof4Of6(t *testing.T) {
	sides, err := generateHashSides(3, 6)
	if err != nil {
		panic(err)
	}

	assert.Equal(t,
		[]bool{true, true, false},
		sides,
	)
}

func TestGenerateHashSidesProof5Of6(t *testing.T) {
	sides, err := generateHashSides(4, 6)
	if err != nil {
		panic(err)
	}

	assert.Equal(t,
		[]bool{false, true},
		sides,
	)
}

func TestGenerateHashSidesProof5Of8(t *testing.T) {
	sides, err := generateHashSides(4, 8)
	if err != nil {
		panic(err)
	}

	assert.Equal(t,
		[]bool{false, false, true},
		sides,
	)
}

func TestGenerateHashSidesLeafPositionTooHigh(t *testing.T) {
	sides, err := generateHashSides(6, 6)
	if sides != nil {
		panic(fmt.Errorf("non-nil sides when leaf position is too high"))
	}

	assert.Equal(t,
		err.Error(),
		"leaf position 6 is too high in proof with 6 leaves",
	)
}
