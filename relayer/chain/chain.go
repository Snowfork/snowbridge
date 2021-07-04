// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package chain

type Message interface{}

// Message from ethereum
type EthereumOutboundMessage struct {
	Call string
	Args []interface{}
}

type Header struct {
	HeaderData interface{}
	ProofData  interface{}
}
