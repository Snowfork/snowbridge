package parachain

import (
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/decred/base58"
	"golang.org/x/crypto/blake2b"
)

func SS58Encode(pubKeyHex string, ss58Prefix uint8) (string, error) {
	if strings.HasPrefix(pubKeyHex, "0x") {
		pubKeyHex = pubKeyHex[2:]
	}

	pubKey, err := hex.DecodeString(pubKeyHex)
	if err != nil {
		return "", fmt.Errorf("failed to decode hex: %w", err)
	}

	address := append([]byte{ss58Prefix}, pubKey...)

	hashInput := append([]byte("SS58PRE"), address...)

	hash := blake2b.Sum512(hashInput)
	checksum := hash[:2]

	fullAddress := append(address, checksum...)

	ss58Addr := base58.Encode(fullAddress)
	return ss58Addr, nil
}
