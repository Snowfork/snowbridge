package types

// Packet contains an application's unique identifier and a substrate-compatible message
type Packet struct {
	AppID   [32]byte
	Message interface{}
}

// NewPacket initializes a new instance of Packet
func NewPacket(appID [32]byte, message interface{}) Packet {
	return Packet{
		AppID:   appID,
		Message: message,
	}
}
