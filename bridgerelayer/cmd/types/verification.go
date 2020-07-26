package types

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
