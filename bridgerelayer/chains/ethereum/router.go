package ethereum

import (
	"bytes"

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

	appAddress := eventData.Contract.Bytes()
	var appID [32]byte
	copy(appID[:], appAddress)

	packet, err := er.buildPacket(eventData.Contract, eventData.Data)
	if err != nil {
		return err
	}

	err = er.sendPacket(appID, packet)
	if err != nil {
		return err
	}

	return nil
}

// BuildPacket builds a data packet from tx data
func (er Router) buildPacket(id common.Address, eLog ctypes.Log) (types.PacketV2, error) {
	// RLP encode event log's Address, Topics, and Data
	var buff bytes.Buffer
	err := eLog.EncodeRLP(&buff)
	if err != nil {
		return types.PacketV2{}, err
	}

	// Generate a proof by signing a hash of the encoded data
	proof, err := prover.GenerateProof(buff.Bytes(), er.keybase.PrivateKey())
	if err != nil {
		return types.PacketV2{}, err
	}

	packet := types.PacketV2{
		Data: buff.Bytes(),
		Signature: proof.Signature,
	}
	return packet, nil
}

// SendPacket sends a tx data packet to the bridge
func (er Router) sendPacket(appID [32]byte, packet types.PacketV2) error {
	log.Info("Sending packet:\n", packet)

	client, err := substrate.NewClient()
	if err != nil {
		panic(err)
	}
	client.SubmitExtrinsic(appID, packet)

	return nil
}
