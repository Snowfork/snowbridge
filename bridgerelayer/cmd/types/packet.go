package types

import (
	"fmt"
)

// AppID ...
type AppID [32]byte

func (r AppID) Hex() string {
	return fmt.Sprintf("%x", r)
}

// Message ...
type Message struct {
	Contents         TxData        // Ethereum chain data including RLP-encoded transaction data
	VerificationData []interface{} // Data used to verify the transaction by the verification module
}

// NewMessage ...
func NewMessage(txData TxData, verificationData []byte) Message {
	return Message{
		Contents:         txData,
		VerificationData: []interface{}{verificationData},
	}
}

// Packet ...
type Packet struct {
	AppID   AppID
	Message Message
}

// NewPacket ...
func NewPacket(appID AppID, message Message) Packet {
	return Packet{
		AppID:   appID,
		Message: message,
	}
}
