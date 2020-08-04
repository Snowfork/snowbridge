package ethereum

import (
	"bytes"
	"os"

	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"

	keybase "github.com/snowfork/polkadot-ethereum/bridgerelayer/keybase/ethereum"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/substrate"
	"github.com/snowfork/polkadot-ethereum/prover"
)

// Router packages raw event data as Packets and relays them to the bridge
type Router struct {
	keybase *keybase.Keypair
}

// NewRouter initializes a new instance of Router
func NewRouter(keybase *keybase.Keypair) Router {
	return Router{
		keybase: keybase,
	}
}

// Route packages tx data as a packet and relays it to the bridge
func (er Router) Route(eventData types.EventData) error {
	packet, err := er.buildPacket(eventData.Contract, eventData.Data)
	if err != nil {
		return err
	}

	err = er.sendPacket(packet)
	if err != nil {
		return err
	}

	return nil
}

// BuildPacket builds a data packet from tx data
func (er Router) buildPacket(id common.Address, eLog ctypes.Log) (types.Packet, error) {
	appAddress := id.Bytes()
	var appID [32]byte
	copy(appID[:], appAddress)

	// RLP encode event log's Address, Topics, and Data
	var buff bytes.Buffer
	err := eLog.EncodeRLP(&buff)
	if err != nil {
		return types.Packet{}, err
	}
	f, err := os.Create("/tmp/log.rlp")
	buff.WriteTo(f)
	f.Close()


	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(buff.Bytes(), er.keybase.PrivateKey())
	if err != nil {
		return types.Packet{}, err
	}

	// Construct and wrap message
	message := types.NewMessage(buff.Bytes(), proof)
	wrappedMessage := types.NewLightClientProof(message)

	packet := types.NewPacket(appID, wrappedMessage)
	return packet, nil
}

// SendPacket sends a tx data packet to the bridge
func (er Router) sendPacket(packet types.Packet) error {
	log.Info("Sending packet:\n", packet)

	client, err := substrate.NewClient()
	if err != nil {
		panic(err)
	}
	client.SubmitExtrinsic(packet)

	return nil
}
