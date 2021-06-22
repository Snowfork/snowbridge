package ethereum

import (
	"bytes"

	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

func MakeTrie(items types.Receipts) (*trie.Trie, error) {
	keyBuf := new(bytes.Buffer)
	trie := new(trie.Trie)

	for i := 0; i < items.Len(); i++ {
		keyBuf.Reset()
		rlp.Encode(keyBuf, uint(i))

		value, err := rlp.EncodeToBytes(items[i])
		if err != nil {
			return nil, err
		}

		trie.Update(keyBuf.Bytes(), value)
	}
	return trie, nil
}
