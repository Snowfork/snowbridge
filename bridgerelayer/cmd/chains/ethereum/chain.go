package ethereum

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/chains"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

var _ types.Chain = &EthChain{}

// EthChain ...
type EthChain struct {
	Config   *chains.ChainConfig // The config of the chain
	Core     *chains.Core        // The chains connection
	Streamer *chains.Streamer    // The streamer of this chain
	Router   *chains.Router      // The router of this chain
	Stop     chan<- int
}

// Initialize ...
func Initialize(chainCfg *types.ChainConfig, sysErr chan<- error) (*EthChain, error) {
	cfg, err := parseChainConfig(chainCfg)
	if err != nil {
		return nil, err
	}

	kpI, err := keystore.KeypairFromAddress(cfg.from, keystore.EthChain, cfg.keystorePath, chainCfg.Insecure)
	if err != nil {
		return nil, err
	}
	kp, _ := kpI.(*secp256k1.Keypair)

	stop := make(chan int)
	conn := connection.NewConnection(cfg.endpoint, cfg.http, kp, logger, cfg.gasLimit, cfg.maxGasPrice)
	err = conn.Connect()
	if err != nil {
		return nil, err
	}

	if chainCfg.LatestBlock {
		curr, err := conn.LatestBlock()
		if err != nil {
			return nil, err
		}
		cfg.startBlock = curr
	}

	streamer := NewEthereumStreamer(conn, cfg, logger, bs, stop, sysErr)
	router := NewEthereumRouter(conn, cfg, logger, stop, sysErr)

	return &Chain{
		cfg:      chainCfg,
		conn:     conn,
		writer:   writer,
		listener: listener,
		stop:     stop,
	}, nil
}

// Start ...
func (c *Chain) Start() error {
	err := c.listener.start()
	if err != nil {
		return err
	}

	err = c.writer.start()
	if err != nil {
		return err
	}

	c.writer.log.Debug("Successfully started chain")
	return nil
}
