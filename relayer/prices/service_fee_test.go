package prices

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func serviceFeeServer(t *testing.T, response string, gotID *string) *httptest.Server {
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, err := io.ReadAll(r.Body)
		require.NoError(t, err)
		var req struct {
			Variables map[string]string `json:"variables"`
		}
		require.NoError(t, json.Unmarshal(body, &req))
		if gotID != nil {
			*gotID = req.Variables["id"]
		}
		_, _ = w.Write([]byte(response))
	}))
}

func TestServiceFeeToEthereumWei(t *testing.T) {
	var gotID string
	server := serviceFeeServer(t, `{"data":{"transferStatusToEthereumV2s":[{"serviceFee":"35000000000000"}]}}`, &gotID)
	defer server.Close()

	fee, err := NewServiceFeeClient(server.URL).ServiceFeeToEthereumWei(context.Background(), "0xABCD")
	require.NoError(t, err)
	assert.Equal(t, "35000000000000", fee.String())
	assert.Equal(t, "0xabcd", gotID)
}

func TestServiceFeeToEthereumWeiNoTransferOrFee(t *testing.T) {
	for _, response := range []string{
		`{"data":{"transferStatusToEthereumV2s":[]}}`,
		`{"data":{"transferStatusToEthereumV2s":[{"serviceFee":null}]}}`,
	} {
		server := serviceFeeServer(t, response, nil)
		fee, err := NewServiceFeeClient(server.URL).ServiceFeeToEthereumWei(context.Background(), "0x01")
		server.Close()
		require.NoError(t, err)
		assert.Equal(t, int64(0), fee.Int64())
	}
}

func TestServiceFeeToEthereumWeiGraphQLError(t *testing.T) {
	server := serviceFeeServer(t, `{"errors":[{"message":"Cannot query field \"serviceFee\""}]}`, nil)
	defer server.Close()

	_, err := NewServiceFeeClient(server.URL).ServiceFeeToEthereumWei(context.Background(), "0x01")
	assert.ErrorContains(t, err, "serviceFee")
}
