package substrate

import (
	"github.com/snowfork/polkadot-ethereum/bridgerelayer/types"
)

// SubstrateRouter ...
type SubstrateRouter struct {
	types.Router
}

// BuildPacket ...
func (sr *SubstrateRouter) BuildPacket(tx []byte, block []byte) (types.Packet, error) {
	// Build packet from substrate transaction data
}

// SendPacket ...
func (sr *SubstrateRouter) SendPacket(packet types.Packet) error {
	// Send packet to bridge...
}
