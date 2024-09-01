package parachain_test

import (
	"fmt"
	"testing"
)

// 100 messages distrubuted to 10 relayers, check waitingPeriod for each relayer
func TestModuloSchedule(t *testing.T) {
	message_count := 100
	total_count := 10
	var waitingPeriod uint64
	for nonce := 1; nonce < message_count; nonce++ {
		for id := 0; id < total_count; id++ {
			waitingPeriod = uint64((nonce + total_count - id) % total_count)
			fmt.Printf("algorithm 1: relay %d waiting for nonce %d for %d\n", id, nonce, waitingPeriod)
		}
	}
	for nonce := 1; nonce < message_count; nonce++ {
		for id := 0; id < total_count; id++ {
			modNonce := nonce % total_count
			if modNonce > id {
				waitingPeriod = uint64(modNonce - id)
			} else {
				waitingPeriod = uint64(id - modNonce)
			}
			fmt.Printf("algorithm 2: relay %d waiting for nonce %d for %d\n", id, nonce, waitingPeriod)
		}
	}

}
