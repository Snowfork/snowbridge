// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package relaychain_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/sirupsen/logrus"

	"github.com/snowfork/go-substrate-rpc-client/v3/scale"
	"github.com/snowfork/go-substrate-rpc-client/v3/types"
	"github.com/snowfork/go-substrate-rpc-client/v3/xxhash"
	"github.com/snowfork/polkadot-ethereum/relayer/chain/relaychain"
)

func TestConnect(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := relaychain.NewConnection("ws://127.0.0.1:9944/", log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()
}


type OptionHeadData struct {
	Option
	Value types.Bytes
}

func (o *OptionHeadData) Decode(decoder scale.Decoder) error {
	return decoder.DecodeOption(&o.hasValue, &o.Value)
}

type Option struct {
	hasValue bool
}

func (o Option) IsNone() bool {
	return !o.hasValue
}

func (o Option) IsSome() bool {
	return o.hasValue
}

// Creates a storage key prefix for a map
func CreateStorageKeyPrefix(prefix, method string) []byte {
	return append(xxhash.New128([]byte(prefix)).Sum(nil), xxhash.New128([]byte(method)).Sum(nil)...)
}

func TestGetAllParaheadsWithOwn(t *testing.T) {
	log := logrus.NewEntry(logrus.New())

	conn := relaychain.NewConnection("wss://polkadot-rpc.snowbridge.network", log)
	err := conn.Connect(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	conn.Close()

	blockHash, err := conn.GetAPI().RPC.Chain.GetFinalizedHead()
	if err != nil {
		t.Fatal(err)
	}

	storageKeyPrefix := CreateStorageKeyPrefix("Paras", "Heads")
	fmt.Printf("Prefix: %#x\n", storageKeyPrefix)

	storageKeys, err := conn.GetAPI().RPC.State.GetKeys(storageKeyPrefix, blockHash)
	if err != nil {
		t.Fatal(err)
	}

	for _, key := range storageKeys {
		fmt.Printf("Key: %#x\n", key)
	}

	heads := []OptionHeadData{}


	ok, err := conn.GetAPI().RPC.State.GetStorageLatest(storageKeys[0], &heads)
	if err != nil {
		t.Fatal(err)
	}

	if !ok {
		t.Fatal(ok)
	}
}

