package beefy

import (
	"fmt"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/util"
)

func TestCleanSignatureNochange(t *testing.T) {
	r, err := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, err := util.HexStringTo32Bytes("0x6dc0d1a7743c3328bfcfe05a2f8691e114f9143776a461ddad6e8b858bb19c1d")
	v := byte(28)
	if err != nil {
		return
	}
	var input []byte
	input = append(r[:], s[:]...)
	input = append(input, v)
	var signature types.BeefySignature
	copy(signature[:], input)
	fmt.Println(signature)
	_v, r, s, err := cleanSignature(signature)
	fmt.Println(_v)
	fmt.Println(util.BytesToHexString(r[:]))
	fmt.Println(util.BytesToHexString(s[:]))
}

func TestCleanSignatureWithRConverted(t *testing.T) {
	r, err := util.HexStringTo32Bytes("0xc1d9e2b5dd63860d27c38a8b276e5a5ab5e19a97452b0cb24094613bcbd517d8")
	s, err := util.HexStringTo32Bytes("0x923f2e588bc3ccd740301fa5d0796e1da5b5c8af38a43e5e1263d3074484a524")
	v := byte(27)
	if err != nil {
		return
	}
	var input []byte
	input = append(r[:], s[:]...)
	input = append(input, v)
	var signature types.BeefySignature
	copy(signature[:], input)
	fmt.Println(signature)
	_v, r, s, err := cleanSignature(signature)
	fmt.Println(_v)
	fmt.Println(util.BytesToHexString(r[:]))
	fmt.Println(util.BytesToHexString(s[:]))
}
