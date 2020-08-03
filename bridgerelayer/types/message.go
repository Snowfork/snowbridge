package types

// Message contains RLP-encoded transaction data and transaction verification data
type Message struct {
	Contents         []byte      // RLP-encoded transaction data
	VerificationData interface{} // Data used to verify the transaction by the verification module
}

// NewMessage initializes a new instance of Message
func NewMessage(txData []byte, verificationData interface{}) Message {
	return Message{
		Contents:         txData,
		VerificationData: verificationData,
	}
}

// Unverified is a wrapper around messages without verification data
type Unverified struct {
	Message
}

// NewUnverified initializes a new instance of Unverified
func NewUnverified(message Message) Unverified {
	return Unverified{Message: message}
}

// ThresholdSignature is a wrapper around messages containing threshold signature verification data
type ThresholdSignature struct {
	Message
}

// NewThresholdSignature initializes a new instance of ThresholdSignature
func NewThresholdSignature(message Message) ThresholdSignature {
	return ThresholdSignature{Message: message}
}

// LightClientProof is a wrapper around messages containing light client proof verification data
type LightClientProof struct {
	Message
}

// NewLightClientProof initializes a new instance of LightClientProof
func NewLightClientProof(message Message) LightClientProof {
	return LightClientProof{Message: message}
}
