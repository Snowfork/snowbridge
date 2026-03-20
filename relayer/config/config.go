package config

import "errors"

type PolkadotConfig struct {
	Endpoint      string `mapstructure:"endpoint"`
	HeartbeatSecs uint64 `mapstructure:"heartbeat-secs"`
}

type ParachainConfig struct {
	Endpoint      string `mapstructure:"endpoint"`
	HeartbeatSecs uint64 `mapstructure:"heartbeat-secs"`
}

type EthereumConfig struct {
	Endpoint             string `mapstructure:"endpoint"`
	// FallbackEndpoint optional second JSON-RPC URL (e.g. public relay) used when the primary endpoint fails with a transient error (timeout / 504).
	FallbackEndpoint     string `mapstructure:"fallback-endpoint"`
	GasFeeCap            uint64 `mapstructure:"gas-fee-cap"`
	GasTipCap            uint64 `mapstructure:"gas-tip-cap"`
	// GasFeeBumpNumerator / GasFeeBumpDenominator scale EIP-1559 caps after eth_estimateGas-style suggestion
	// (SuggestGasTipCap and feeCap = tip + 2*baseFee, matching go-ethereum bind). Example: 130/100 = +30%.
	// When both are zero, defaults to 130/100. Use 100/100 for no extra bump beyond the bind formula.
	GasFeeBumpNumerator   uint64 `mapstructure:"gas-fee-bump-numerator"`
	GasFeeBumpDenominator uint64 `mapstructure:"gas-fee-bump-denominator"`
	GasLimit              uint64 `mapstructure:"gas-limit"`
	HeartbeatSecs         uint64 `mapstructure:"heartbeat-secs"`
	PendingTxTimeoutSecs  uint64 `mapstructure:"pending-tx-timeout-secs"`
}

type OFACConfig struct {
	Enabled bool   `mapstructure:"enabled"`
	ApiKey  string `mapstructure:"apiKey"`
}

func (p ParachainConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] is not set")
	}
	return nil
}

func (e EthereumConfig) Validate() error {
	if e.Endpoint == "" {
		return errors.New("[endpoint] config is not set")
	}
	return nil
}

func (p PolkadotConfig) Validate() error {
	if p.Endpoint == "" {
		return errors.New("[endpoint] config is not set")
	}
	return nil
}

func (o OFACConfig) Validate() error {
	if o.Enabled && o.ApiKey == "" {
		return errors.New("OFAC is enabled but no [apiKey] set")
	}
	return nil
}
