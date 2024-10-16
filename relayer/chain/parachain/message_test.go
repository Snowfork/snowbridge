package parachain

import (
	"testing"

	gethCommon "github.com/ethereum/go-ethereum/common"
	assert "github.com/stretchr/testify/require"
)

func TestGetDestination(t *testing.T) {
	registerTokenPayload := "00a736aa000000000000774667629726ec1fabebcec0d9139bd1c8f72a2300e87648170000000000000000000000"
	decodePayloadAndCompareDestinationAddress(t, registerTokenPayload, "") // register token does not have a destination

	sendTokenPayload := "00a736aa000000000001774667629726ec1fabebcec0d9139bd1c8f72a23008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a4800c16ff2862300000000000000000000e87648170000000000000000000000"
	bobAddress := "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
	decodePayloadAndCompareDestinationAddress(t, sendTokenPayload, bobAddress)

	sendTokenToPayload := "00a736aa000000000001774667629726ec1fabebcec0d9139bd1c8f72a2301d00700001cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c00286bee000000000000000000000000000064a7b3b6e00d000000000000000000e87648170000000000000000000000"
	ferdieAddress := "0x1cbd2d43530a44705ad088af313e18f80b53ef16b36177cd4b77b846f2a5f07c"
	decodePayloadAndCompareDestinationAddress(t, sendTokenToPayload, ferdieAddress)

	sendNativeTokenPayload := "00a736aa0000000000022121cfe35065c0c33465fbada265f08e9613428a4b9eb4bb717cd7db2abf622e008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48065cd1d00000000000000000000000000e87648170000000000000000000000"
	decodePayloadAndCompareDestinationAddress(t, sendNativeTokenPayload, bobAddress)
}

func decodePayloadAndCompareDestinationAddress(t *testing.T, payload, expectedAddress string) {
	data := gethCommon.Hex2Bytes(payload)

	destination, err := GetDestination(data)
	assert.NoError(t, err)

	assert.Equal(t, expectedAddress, destination)
}
