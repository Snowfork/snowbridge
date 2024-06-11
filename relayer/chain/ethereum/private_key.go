package ethereum

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/snowfork/snowbridge/relayer/crypto/secp256k1"
	"github.com/snowfork/snowbridge/relayer/secrets"
)

func ResolvePrivateKey(privateKey, privateKeyFile, privateKeyID string) (*secp256k1.Keypair, error) {
	switch {
	case privateKey != "":
	case privateKeyFile != "":
		contentBytes, err := os.ReadFile(privateKeyFile)
		if err != nil {
			return nil, fmt.Errorf("load private key: %w", err)
		}
		privateKey = strings.TrimSpace(string(contentBytes))
	case privateKeyID != "":
		secret, err := secrets.GetSecretValue(context.TODO(), privateKeyID)
		if err != nil {
			return nil, err
		}
		privateKey = secret
	default:
		return nil, fmt.Errorf("Unable to resolve a private key")
	}

	keypair, err := secp256k1.NewKeypairFromString(strings.TrimPrefix(privateKey, "0x"))
	if err != nil {
		return nil, fmt.Errorf("parse private key: %w", err)
	}

	return keypair, nil
}
