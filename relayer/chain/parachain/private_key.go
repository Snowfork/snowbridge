package parachain

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/snowfork/go-substrate-rpc-client/v4/signature"
	"github.com/snowfork/snowbridge/relayer/secrets"
)

func ResolvePrivateKey(privateKey, privateKeyFile, privateKeyID string) (*signature.KeyringPair, error) {
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

	keypair, err := signature.NewEcdsaKeyringPair(privateKey)
	if err != nil {
		return nil, fmt.Errorf("parse private key: %w", err)
	}

	return &keypair, nil
}
