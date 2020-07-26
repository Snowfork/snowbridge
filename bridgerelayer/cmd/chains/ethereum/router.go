package ethereum

import (
	"bytes"

	"github.com/ethereum/go-ethereum/common"
	ctypes "github.com/ethereum/go-ethereum/core/types"
	log "github.com/sirupsen/logrus"

	"github.com/snowfork/polkadot-ethereum/bridgerelayer/cmd/types"
)

// Router packages raw event data as Packets and relays them to the bridge
type Router struct{}

// NewRouter initializes a new instance of Router
func NewRouter() Router {
	return Router{}
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
		log.Info(err)
	}

	verificationData := []byte("TODO: get via SignatureProver component")
	message := types.NewMessage(buff.Bytes(), verificationData)
	wrappedMessage := types.NewUnverified(message)

	packet := types.NewPacket(appID, wrappedMessage)

	return packet, nil
}

// SendPacket sends a tx data packet to the bridge
func (er Router) sendPacket(packet types.Packet) error {
	log.Info("Sending packet:\n", packet)

	// Bridge.Send(packet.AppID, packet.Message)

	return nil
}
