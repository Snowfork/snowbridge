package ethereum_test

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/rlp"
	gethTrie "github.com/ethereum/go-ethereum/trie"
	"github.com/sirupsen/logrus/hooks/test"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/ethereum"
	"github.com/snowfork/polkadot-ethereum/relayer/substrate"
	"github.com/stretchr/testify/assert"
)

type TestProof substrate.ProofData

// For interface gethTrie.KeyValueReader
func (tp *TestProof) Get(key []byte) ([]byte, error) {
	for i, k := range tp.Keys {
		if bytes.Equal(k, key) {
			return tp.Values[i], nil
		}
	}
	return nil, fmt.Errorf("Value for key %s does not exist", key)
}

// For interface gethTrie.KeyValueReader
func (tp *TestProof) Has(key []byte) (bool, error) {
	_, err := tp.Get(key)
	return err == nil, nil
}

func TestMessage_Proof(t *testing.T) {
	block := block11408438()
	receipts := receipts11408438()
	// We'll prove inclusion for this event by proving inclusion for
	// the encapsulating receipt

	event5_5 := receipts[5].Logs[5]

	receipt5Encoded, err := rlp.EncodeToBytes(receipts[5])
	if err != nil {
		panic(err)
	}

	// Construct Merkle Patricia Trie for receipts
	keybuf := new(bytes.Buffer)
	receiptTrie := new(gethTrie.Trie)
	for i := 0; i < receipts.Len(); i++ {
		keybuf.Reset()
		rlp.Encode(keybuf, uint(i))
		receiptTrie.Update(keybuf.Bytes(), receipts.GetRlp(i))
	}
	if receiptTrie.Hash() != block.ReceiptHash() {
		panic("Receipt trie does not match block receipt hash")
	}

	logger, _ := test.NewNullLogger()
	mapping := make(map[common.Address]string)
	mapping[event5_5.Address] = "InboundChannel.submit"

	msg, err := ethereum.MakeMessageFromEvent(mapping, event5_5, receiptTrie, logger.WithField("test", "ing"))
	assert.Nil(t, err)
	assert.NotNil(t, msg)

	msgInner, ok := msg.Args[0].(substrate.Message)
	if !ok {
		panic("unexpected type")
	}

	assert.Equal(t, block.Hash().Hex(), msgInner.Proof.BlockHash.Hex())
	key, err := rlp.EncodeToBytes(uint(msgInner.Proof.TxIndex))
	if err != nil {
		panic(err)
	}
	proofNodes := TestProof(*msgInner.Proof.Data)
	provenReceipt, err := gethTrie.VerifyProof(block.ReceiptHash(), key, &proofNodes)
	assert.Nil(t, err)
	assert.Equal(t, provenReceipt, receipt5Encoded)
}
