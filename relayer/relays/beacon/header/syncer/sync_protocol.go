package syncer

import (
	"encoding/hex"
	"fmt"
	"strconv"
	"strings"
)

func (s *Syncer) ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / (s.SlotsInEpoch * s.EpochsPerSyncCommitteePeriod)
}

func (s *Syncer) ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / s.SlotsInEpoch
}

func (s *Syncer) IsStartOfEpoch(slot uint64) bool {
	return slot%s.SlotsInEpoch == 0
}

func (s *Syncer) CalculateNextCheckpointSlot(slot uint64) uint64 {
	syncPeriod := s.ComputeSyncPeriodAtSlot(slot)

	// on new period boundary
	if syncPeriod*s.SlotsInEpoch*s.EpochsPerSyncCommitteePeriod == slot {
		return slot
	}

	return (syncPeriod + 1) * s.SlotsInEpoch * s.EpochsPerSyncCommitteePeriod
}

func hexToBinaryString(rawHex string) string {
	hexString := strings.Replace(rawHex, "0x", "", -1)

	// Chunkify strings into array of strings of 8 characters long (to ParseUint safely below)
	chunkSize := 8

	resultStr := ""
	chunks := []string{}
	for i, r := range hexString {
		resultStr = resultStr + string(r)
		if i > 0 && (i+1)%chunkSize == 0 {
			chunks = append(chunks, resultStr)
			resultStr = ""
		}
	}

	// If there was a remainder, add the last string to the chunks as well.
	if resultStr != "" {
		chunks = append(chunks, resultStr)
	}

	// Convert chunks into binary string
	binaryStr := ""
	for _, str := range chunks {
		i, err := strconv.ParseUint(str, 16, 32)
		if err != nil {
			fmt.Printf("%s", err)
		}
		binaryStr = binaryStr + fmt.Sprintf("%b", i)
	}

	return binaryStr
}

func hexStringToPublicKey(hexString string) ([48]byte, error) {
	var pubkeyBytes [48]byte
	key, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return [48]byte{}, err
	}

	copy(pubkeyBytes[:], key)

	return pubkeyBytes, nil
}

func hexStringToByteArray(hexString string) ([]byte, error) {
	bytes, err := hex.DecodeString(strings.Replace(hexString, "0x", "", 1))
	if err != nil {
		return []byte{}, err
	}

	return bytes, nil
}
