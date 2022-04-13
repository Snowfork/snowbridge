package syncer

import (
	"fmt"
	"strconv"
	"strings"
)

const SLOTS_IN_EPOCH uint64 = 32

const EPOCHS_PER_SYNC_COMMITTEE_PERIOD uint64 = 256

type Syncer struct {
	Client BeaconClient
	Cache  BeaconCache
}

func New(endpoint string) *Syncer {
	return &Syncer{
		Client: *NewBeaconClient(endpoint),
		Cache:  *NewBeaconCache(),
	}
}

func ComputeEpochAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func ComputeEpochForNextPeriod(epoch uint64) uint64 {
	return epoch + (EPOCHS_PER_SYNC_COMMITTEE_PERIOD - (epoch % EPOCHS_PER_SYNC_COMMITTEE_PERIOD))
}

func ComputeSyncPeriodAtSlot(slot uint64) uint64 {
	return slot / SLOTS_IN_EPOCH
}

func ComputeSyncPeriodAtEpoch(epoch uint64) uint64 {
	return epoch / EPOCHS_PER_SYNC_COMMITTEE_PERIOD
}

func HexToBinaryString(rawHex string) string {
	hex := strings.Replace(rawHex, "0x", "", -1)

	// Chunkify strings into array of strings of 8 characters long (to ParseUint safely below)
	chunkSize := 8

	resultStr := ""
	chunks := []string{}
	for i, r := range hex {
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
