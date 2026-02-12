package errors

import "errors"

// ErrProofNotReady is returned when the proof is not yet cached and the client should retry
var ErrProofNotReady = errors.New("proof not ready, please retry")
