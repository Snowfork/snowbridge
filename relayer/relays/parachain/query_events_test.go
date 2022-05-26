package parachain_test

import (
	"context"
	"io/ioutil"
	"os"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v4/types"
	"github.com/snowfork/snowbridge/relayer/relays/parachain"
	"github.com/stretchr/testify/assert"
)

var mock = `{
  "items": [
    {
      "id": 0,
	  "hash": "0x82a6824ed57e7bb78b51673803f49de2e8373dbaf6da12e0f3f99f4d9779459a",
	  "data": "0x040400b8ea8cb425d85536b158d661da1ef0895bb92f1d91017ed9db59d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d00000000000000000000000089b4ab1ef20763630df9743acf155865600daff20000000000000000000000000000000000000000000000056bc75e2d63100000"
	}
  ]
}
`

func TestQueryEvents(t *testing.T) {
    tmpFile, err := ioutil.TempFile(os.TempDir(), "test-query-events-")
    if err != nil {
        t.Fatal(err)
    }
	defer os.Remove(tmpFile.Name())

    if _, err = tmpFile.Write([]byte(mock)); err != nil {
        t.Fatal(err)
    }

    if err := tmpFile.Close(); err != nil {
        t.Fatal(err)
    }

	client := parachain.NewQueryClient()
	client.NameArgs = func(_ string, _ string) (string, []string) {
		return "cat", []string{tmpFile.Name()}
	}

	foo, _ := types.NewHashFromHexString("0x6456d3a2f0c7526d63ad50e79dc8a462931a58ffd57270c3c8aabbcdbd78e76b")
	events, err := client.QueryEvents(context.Background(), "", foo)
	if err != nil {
		t.Fatal(err)
	}

	assert.NotNil(t, events.Basic)
	assert.Equal(t, events.Basic.Bundle.Nonce.Int64(), int64(1))
	assert.Equal(t, len(events.Basic.Bundle.Messages), 1)
}
