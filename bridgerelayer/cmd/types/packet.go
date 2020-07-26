package types

// Message contains RLP-encoded transaction data and transaction verification data
type Message struct {
	Contents         []byte // RLP-encoded transaction data
	VerificationData []byte // Data used to verify the transaction by the verification module
}

// NewMessage initializes a new instance of Message
func NewMessage(txData, verificationData []byte) Message {
	return Message{
		Contents:         txData,
		VerificationData: verificationData,
	}
}

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
