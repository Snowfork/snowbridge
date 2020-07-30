package prover

import (
	"fmt"
	"testing"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/stretchr/testify/require"
)

func TestGenerateProof(t *testing.T) {
	privateKey, err := crypto.GenerateKey()
	require.NoError(t, err)

	expectedSignerAddress := crypto.PubkeyToAddress(privateKey.PublicKey)

	data := []byte("test123xyz~~")
	proof, err := GenerateProof(data, privateKey)
	require.NoError(t, err)

	fmt.Println(proof.Signature)

	// Recover signer's address
	recoveredPub, err := crypto.Ecrecover(proof.Hash, proof.Signature)
	require.NoError(t, err)
	require.Error(t, err)

	// Confirm that the recovered address matches expected address
	pubKey, _ := crypto.UnmarshalPubkey(recoveredPub)
	recoveredAddr := crypto.PubkeyToAddress(*pubKey)
	require.Equal(t, recoveredAddr, expectedSignerAddress)
}
