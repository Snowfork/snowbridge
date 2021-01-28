package substrate

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/snowfork/go-substrate-rpc-client/v2/scale"
	"github.com/stretchr/testify/assert"
)

func encodeToBytes(value interface{}) ([]byte, error) {
	var buffer = bytes.Buffer{}
	err := scale.NewEncoder(&buffer).Encode(value)
	if err != nil {
		return buffer.Bytes(), err
	}
	return buffer.Bytes(), nil
}

func decodeFromBytes(bz []byte, target interface{}) error {
	return scale.NewDecoder(bytes.NewReader(bz)).Decode(target)
}

func TestMessage_EncodeDecode(t *testing.T) {

	input := Message{
		Data: []byte{0, 1, 2},
		VerificationInput: VerificationInput{
			IsBasic: true,
			AsBasic: VerificationBasic{
				BlockNumber: 938,
				EventIndex:  4,
			},
		},
	}

	encoded, err := encodeToBytes(input)
	if err != nil {
		panic(err)
	}

	fmt.Println("length: ", len(encoded))

	var decoded Message
	err = decodeFromBytes(encoded, &decoded)
	if err != nil {
		panic(err)
	}

	assert.Equal(t, input, decoded, "The two messages should be the same")
}
