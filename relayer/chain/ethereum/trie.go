package ethereum

import (
	"bytes"

	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

func MakeTrie(items types.Receipts) *trie.Trie {
	keyBuf := new(bytes.Buffer)
	valueBuf := new(bytes.Buffer)
	trie := new(trie.Trie)

	for i := 0; i < items.Len(); i++ {
		keyBuf.Reset()
		valueBuf.Reset()

		rlp.Encode(keyBuf, uint(i))
		items.EncodeIndex(i, valueBuf)
		trie.Update(keyBuf.Bytes(), valueBuf.Bytes())
	}
	return trie
}
