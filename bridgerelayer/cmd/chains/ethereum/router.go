package ethereum

import (
	"bytes"
	"context"
	"log"

	ethTypes "github.com/ethereum/go-ethereum/core/types"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

// EthRouter ...
type EthRouter struct{}

// BuildPacket ...
func (er *EthRouter) BuildPacket(tx ethTypes.Transaction, block ethTypes.Block) (types.Packet, error) {
	chainID, err := client.NetworkID(context.Background())
	if err != nil {
		log.Fatal(err)
		return types.Packet{}, err
	}

	receipt, err := client.TransactionReceipt(context.Background(), tx.Hash())
	if err != nil {
		log.Fatal(err)
		return types.Packet{}, err
	}

	var receiptBuf bytes.Buffer
	receipt.EncodeRLP(&receiptBuf)

	// Transaction data
	txData := types.NewEthTxData(chainID.Bytes(), block.Hash().Bytes(),
		tx.Hash().Bytes(), tx.Data(), receiptBuf.Bytes())

	// Message
	message := types.NewMessage(txData, []byte{})

	// Packet
	var appID types.AppID
	copy(appID[:], tx.To().Bytes())
	packet := types.NewPacket(appID, message)

	return packet, nil
}

// SendPacket ...
func (er *EthRouter) SendPacket(packet types.Packet) error {
	// Send packet to bridge...
}
