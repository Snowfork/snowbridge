package executionV3

import (
	"fmt"

	"github.com/snowfork/snowbridge/relayer/config"
	beaconconf "github.com/snowfork/snowbridge/relayer/relays/beacon/config"
	"github.com/snowfork/snowbridge/relayer/relays/gasestimator"
)

type Config struct {
	Source       SourceConfig      `mapstructure:"source"`
	Sink         SinkConfig        `mapstructure:"sink"`
	IgnoreNonces []uint64          `mapstructure:"ignoreNonces"`
	OFAC         config.OFACConfig `mapstructure:"ofac"`
	// DeadNoncesPath is the file path where nonces that have failed with a
	// permanent error are persisted. When empty, the store is in-memory only
	// and dead nonces will be re-attempted after a relayer restart.
	DeadNoncesPath string `mapstructure:"deadNoncesPath"`
	// Tip (in BridgeHub's native token base units) attached to every submission
	// extrinsic, including the force_batch. Defaults to 0. A non-zero tip raises
	// block-inclusion priority so the batch lands sooner when competing for a slot.
	Tip uint64 `mapstructure:"tip"`
	// MinMessageRelayerFeeWei is the static reward floor on RelayerFee + decoded
	// UI/service fee (from Payload.Xcm). Empty/0 disables the floor.
	MinMessageRelayerFeeWei string `mapstructure:"min-message-relayer-fee-wei"`
	// GasEstimation is optional dry-run profitability. service-fee-recipients here
	// are also used for the static XCM service-fee decode when estimation is off.
	GasEstimation gasestimator.Config `mapstructure:"gasEstimation"`
}

type SourceConfig struct {
	Ethereum  config.EthereumConfig   `mapstructure:"ethereum"`
	Contracts ContractsConfig         `mapstructure:"contracts"`
	Beacon    beaconconf.BeaconConfig `mapstructure:"beacon"`
	// ChannelID is the V1 inbound channel (EthereumInboundQueue). Empty disables V1.
	ChannelID ChannelID `mapstructure:"channel-id"`
}

type ContractsConfig struct {
	Gateway string `mapstructure:"Gateway"`
}

type SinkConfig struct {
	Parachain  beaconconf.ParachainConfig `mapstructure:"parachain"`
	SS58Prefix uint8                      `mapstructure:"ss58Prefix"`
}

type ChannelID [32]byte

func (c Config) V1Enabled() bool {
	return c.Source.ChannelID != [32]byte{}
}

func (c Config) Validate() error {
	err := c.Source.Beacon.Validate()
	if err != nil {
		return fmt.Errorf("beacon config validation: %w", err)
	}
	err = c.Sink.Parachain.Validate()
	if err != nil {
		return fmt.Errorf("parachain config validation: %w", err)
	}
	if c.Source.Contracts.Gateway == "" {
		return fmt.Errorf("source setting [gateway] is not set")
	}
	err = c.OFAC.Validate()
	if err != nil {
		return fmt.Errorf("ofac config: %w", err)
	}
	err = c.GasEstimation.Validate()
	if err != nil {
		return fmt.Errorf("gas estimation config: %w", err)
	}
	return nil
}
