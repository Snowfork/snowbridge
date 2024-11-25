package beefy

import (
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/util"
	"github.com/stretchr/testify/assert"
)

func TestCleanSignatureNochange(t *testing.T) {
	r, err := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, err := util.HexStringTo32Bytes("0x6dc0d1a7743c3328bfcfe05a2f8691e114f9143776a461ddad6e8b858bb19c1d")
	v := byte(28)
	signature := buildSignature(v, r, s)
	vAfter, rAfter, sAfter, err := CleanSignature(signature)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, vAfter, v)
	assert.Equal(t, rAfter, r)
	assert.Equal(t, sAfter, s)

}

func TestCleanSignatureWithSConverted(t *testing.T) {
	r, err := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, err := util.HexStringTo32Bytes("0x923f2e588bc3ccd740301fa5d0796e1da5b5c8af38a43e5e1263d3074484a524")
	v := byte(27)
	signature := buildSignature(v, r, s)

	negativeS, err := util.HexStringTo32Bytes("0x6dc0d1a7743c3328bfcfe05a2f8691e114f9143776a461ddad6e8b858bb19c1d")
	negativeV := byte(28)

	vAfter, rAfter, sAfter, err := CleanSignature(signature)
	if err != nil {
		t.Fatal(err)
	}
	assert.Equal(t, vAfter, negativeV)
	assert.Equal(t, rAfter, r)
	assert.Equal(t, sAfter, negativeS)
}

func buildSignature(v uint8, r [32]byte, s [32]byte) (signature types.BeefySignature) {
	var input []byte
	input = append(r[:], s[:]...)
	input = append(input, v)
	copy(signature[:], input)
	return signature
}
