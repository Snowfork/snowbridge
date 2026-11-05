package prices

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"math/big"
	"net/http"
	"os"
	"strings"
)

// serviceFeeToEthereumQuery reads the ether service fee the dApp deposited on Asset Hub
// for a Polkadot->Ethereum message (snowbridge-subsquid TransferStatusToEthereumV2).
const serviceFeeToEthereumQuery = `query ($id: String!) {
  transferStatusToEthereumV2s(limit: 1, where: {messageId_eq: $id, serviceFeeToken_eq: "ETH"}) {
    serviceFee
  }
}`

type serviceFeeResponse struct {
	Data struct {
		Transfers []struct {
			ServiceFee *string `json:"serviceFee"`
		} `json:"transferStatusToEthereumV2s"`
	} `json:"data"`
	Errors []struct {
		Message string `json:"message"`
	} `json:"errors"`
}

// ServiceFeeClient reads per-message service fees from the snowbridge indexer.
type ServiceFeeClient struct {
	url        string
	httpClient *http.Client
}

func NewServiceFeeClient(url string) *ServiceFeeClient {
	return &ServiceFeeClient{
		url:        url,
		httpClient: &http.Client{Timeout: indexerFetchTimeout},
	}
}

// NewServiceFeeClientFromEnv uses PRICE_INDEXER_API_URL, the same indexer the price
// client reads. It returns nil when the URL is unset.
func NewServiceFeeClientFromEnv() *ServiceFeeClient {
	url := os.Getenv(indexerAPIURLEnvKey)
	if url == "" {
		return nil
	}
	return NewServiceFeeClient(url)
}

// ServiceFeeToEthereumWei returns the ether service fee recorded for the message, or
// zero when the indexer has no transfer or no fee for it.
func (c *ServiceFeeClient) ServiceFeeToEthereumWei(ctx context.Context, messageID string) (*big.Int, error) {
	reqBody, err := json.Marshal(map[string]interface{}{
		"query":     serviceFeeToEthereumQuery,
		"variables": map[string]string{"id": strings.ToLower(messageID)},
	})
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.url, bytes.NewReader(reqBody))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, strings.TrimSpace(string(body)))
	}

	var parsed serviceFeeResponse
	if err := json.Unmarshal(body, &parsed); err != nil {
		return nil, err
	}
	if len(parsed.Errors) > 0 {
		return nil, fmt.Errorf("indexer graphql error: %s", parsed.Errors[0].Message)
	}

	if len(parsed.Data.Transfers) == 0 || parsed.Data.Transfers[0].ServiceFee == nil {
		return new(big.Int), nil
	}
	fee, ok := new(big.Int).SetString(*parsed.Data.Transfers[0].ServiceFee, 10)
	if !ok || fee.Sign() < 0 {
		return nil, fmt.Errorf("invalid service fee %q", *parsed.Data.Transfers[0].ServiceFee)
	}
	return fee, nil
}
