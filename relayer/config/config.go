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
	// SubmitEndpoint is optional and used only for eth_sendRawTransaction (signed txs).
	// Reads / eth_call / receipts / gas stay on Endpoint. Typical value: Flashbots Protect
	// (e.g. https://rpc.flashbots.net/fast) so simulated reverts are dropped unpaid.
	SubmitEndpoint       string `mapstructure:"submit-endpoint"`
	GasFeeCap            uint64 `mapstructure:"gas-fee-cap"`
	GasTipCap            uint64 `mapstructure:"gas-tip-cap"`
	GasLimit             uint64 `mapstructure:"gas-limit"`
	HeartbeatSecs        uint64 `mapstructure:"heartbeat-secs"`
	PendingTxTimeoutSecs uint64 `mapstructure:"pending-tx-timeout-secs"`
	// Headers are sent on every JSON-RPC request to Endpoint (e.g. an X-API-Key for a
	// dry-run/forked-mainnet RPC). Empty means a plain dial with no extra headers.
	// Not applied to SubmitEndpoint (builder RPCs expect a plain dial).
	Headers map[string]string `mapstructure:"headers"`
}

type OFACConfig struct {
	Enabled bool   `mapstructure:"enabled"`
	ApiKey  string `mapstructure:"apiKey"`
	// CacheTTLSecs is how long a clean (not sanctioned) TRM result is cached.
	// Sanctioned hits are cached for the process lifetime. Zero uses the ofac package default.
	CacheTTLSecs uint64 `mapstructure:"cache-ttl-secs"`
}

func (p ParachainConfig) ParseEndpoints() []string {
	return ParseEndpoints(p.Endpoint)
}

func (e EthereumConfig) ParseEndpoints() []string {
	return ParseEndpoints(e.Endpoint)
}

func (p PolkadotConfig) ParseEndpoints() []string {
	return ParseEndpoints(p.Endpoint)
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
