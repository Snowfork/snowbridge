package api

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestUnmarshalBlockResponse(t *testing.T) {
	blockResponse := BeaconBlockResponse{}
	data, _ := os.ReadFile("web/packages/test/testdata/beacon_block_6815804.json")
	if data != nil {
		err := json.Unmarshal(data, &blockResponse)
		assert.Nil(t, err)
	}
}
