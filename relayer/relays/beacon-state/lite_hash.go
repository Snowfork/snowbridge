package beaconstate

import (
	"encoding/binary"

	"github.com/minio/sha256-simd"
)

// hashFixedVector computes the hash tree root of a fixed-size vector of 32-byte elements.
// This is used for StateRoots, RandaoMixes, etc.
func hashFixedVector(data []byte, elementSize, count int) [32]byte {
	if len(data) == 0 {
		return [32]byte{}
	}

	// Build leaves from elements
	leaves := make([][32]byte, count)
	for i := 0; i < count && i*elementSize < len(data); i++ {
		copy(leaves[i][:], data[i*elementSize:(i+1)*elementSize])
	}

	return merkleize(leaves)
}

// hashVariableList computes the hash tree root of a variable-length list.
// The hash includes the length mixed in.
func hashVariableList(data []byte, elementSize int) [32]byte {
	if len(data) == 0 || elementSize == 0 {
		// Empty list: hash of zero mixed with length 0
		return mixInLength([32]byte{}, 0)
	}

	count := len(data) / elementSize
	leaves := make([][32]byte, count)
	for i := 0; i < count; i++ {
		// For each element, compute its hash
		element := data[i*elementSize : (i+1)*elementSize]
		leaves[i] = sha256Hash(element)
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashValidators computes the hash tree root of the validators list.
// Each validator is 121 bytes. We hash them in a streaming fashion.
func hashValidators(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	const validatorSize = 121
	count := len(data) / validatorSize

	// Process validators in chunks to limit memory
	const chunkSize = 1024 // Process 1024 validators at a time
	var leaves [][32]byte

	for i := 0; i < count; i += chunkSize {
		end := i + chunkSize
		if end > count {
			end = count
		}

		for j := i; j < end; j++ {
			validatorData := data[j*validatorSize : (j+1)*validatorSize]
			// Hash each validator's fields according to SSZ spec
			leaves = append(leaves, hashValidator(validatorData))
		}
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashValidator computes the hash of a single validator (121 bytes)
func hashValidator(data []byte) [32]byte {
	if len(data) < 121 {
		return [32]byte{}
	}

	// Validator SSZ layout:
	// pubkey: 48 bytes
	// withdrawal_credentials: 32 bytes
	// effective_balance: 8 bytes
	// slashed: 1 byte
	// activation_eligibility_epoch: 8 bytes
	// activation_epoch: 8 bytes
	// exit_epoch: 8 bytes
	// withdrawable_epoch: 8 bytes

	leaves := make([][32]byte, 8)

	// Field 0: pubkey (48 bytes, padded to 64, then hashed)
	pubkeyPadded := make([]byte, 64)
	copy(pubkeyPadded, data[0:48])
	leaves[0] = sha256Hash(pubkeyPadded)

	// Field 1: withdrawal_credentials (32 bytes)
	copy(leaves[1][:], data[48:80])

	// Field 2: effective_balance (8 bytes, left-padded to 32)
	leaves[2] = uint64ToLeaf(binary.LittleEndian.Uint64(data[80:88]))

	// Field 3: slashed (1 byte bool, left-padded to 32)
	if data[88] != 0 {
		leaves[3][0] = 1
	}

	// Field 4: activation_eligibility_epoch
	leaves[4] = uint64ToLeaf(binary.LittleEndian.Uint64(data[89:97]))

	// Field 5: activation_epoch
	leaves[5] = uint64ToLeaf(binary.LittleEndian.Uint64(data[97:105]))

	// Field 6: exit_epoch
	leaves[6] = uint64ToLeaf(binary.LittleEndian.Uint64(data[105:113]))

	// Field 7: withdrawable_epoch
	leaves[7] = uint64ToLeaf(binary.LittleEndian.Uint64(data[113:121]))

	return merkleize(leaves)
}

// hashBalances computes the hash tree root of the balances list.
// Each balance is a uint64 (8 bytes).
func hashBalances(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	count := len(data) / 8

	// Pack 4 uint64s into each 32-byte chunk
	numChunks := (count + 3) / 4
	leaves := make([][32]byte, numChunks)

	for i := 0; i < numChunks; i++ {
		for j := 0; j < 4; j++ {
			idx := i*4 + j
			if idx < count {
				balance := binary.LittleEndian.Uint64(data[idx*8 : (idx+1)*8])
				binary.LittleEndian.PutUint64(leaves[i][j*8:], balance)
			}
		}
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashSlashings computes the hash tree root of the slashings vector.
// 8192 uint64 values.
func hashSlashings(data []byte) [32]byte {
	const count = 8192

	// Pack 4 uint64s into each 32-byte chunk
	numChunks := count / 4
	leaves := make([][32]byte, numChunks)

	for i := 0; i < numChunks; i++ {
		for j := 0; j < 4; j++ {
			idx := i*4 + j
			if idx*8 < len(data) {
				val := binary.LittleEndian.Uint64(data[idx*8 : (idx+1)*8])
				binary.LittleEndian.PutUint64(leaves[i][j*8:], val)
			}
		}
	}

	return merkleize(leaves)
}

// hashParticipation computes the hash tree root of epoch participation.
// This is a byte list (one byte per validator).
func hashParticipation(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	// Pack 32 bytes into each chunk
	numChunks := (len(data) + 31) / 32
	leaves := make([][32]byte, numChunks)

	for i := 0; i < numChunks; i++ {
		start := i * 32
		end := start + 32
		if end > len(data) {
			end = len(data)
		}
		copy(leaves[i][:], data[start:end])
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(len(data)))
}

// hashInactivityScores computes the hash tree root of inactivity scores.
// List of uint64 values.
func hashInactivityScores(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	count := len(data) / 8

	// Pack 4 uint64s into each 32-byte chunk
	numChunks := (count + 3) / 4
	leaves := make([][32]byte, numChunks)

	for i := 0; i < numChunks; i++ {
		for j := 0; j < 4; j++ {
			idx := i*4 + j
			if idx < count {
				score := binary.LittleEndian.Uint64(data[idx*8 : (idx+1)*8])
				binary.LittleEndian.PutUint64(leaves[i][j*8:], score)
			}
		}
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashHistoricalSummaries computes the hash tree root of historical summaries.
// Each summary is 64 bytes (two 32-byte roots).
func hashHistoricalSummaries(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	const summarySize = 64
	count := len(data) / summarySize
	leaves := make([][32]byte, count)

	for i := 0; i < count; i++ {
		summary := data[i*summarySize : (i+1)*summarySize]
		// Each summary has 2 fields: block_summary_root and state_summary_root
		var leaf1, leaf2 [32]byte
		copy(leaf1[:], summary[0:32])
		copy(leaf2[:], summary[32:64])
		leaves[i] = hashTwo(leaf1, leaf2)
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashExecutionPayloadHeader computes the hash of the execution payload header
func hashExecutionPayloadHeader(data []byte) [32]byte {
	if len(data) == 0 {
		return [32]byte{}
	}
	return sha256Hash(data)
}

// hashPendingDeposits computes the hash tree root of pending deposits.
// Each PendingDeposit is 192 bytes (48 + 32 + 8 + 96 + 8).
func hashPendingDeposits(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	const depositSize = 192
	count := len(data) / depositSize
	leaves := make([][32]byte, count)

	for i := 0; i < count; i++ {
		deposit := data[i*depositSize : (i+1)*depositSize]
		leaves[i] = hashPendingDeposit(deposit)
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashPendingDeposit hashes a single PendingDeposit
func hashPendingDeposit(data []byte) [32]byte {
	if len(data) < 192 {
		return [32]byte{}
	}

	leaves := make([][32]byte, 8)

	// pubkey: 48 bytes, padded to 64
	pubkeyPadded := make([]byte, 64)
	copy(pubkeyPadded, data[0:48])
	leaves[0] = sha256Hash(pubkeyPadded)

	// withdrawal_credentials: 32 bytes
	copy(leaves[1][:], data[48:80])

	// amount: 8 bytes
	leaves[2] = uint64ToLeaf(binary.LittleEndian.Uint64(data[80:88]))

	// signature: 96 bytes, hashed to 32
	leaves[3] = sha256Hash(data[88:184])

	// index: 8 bytes
	leaves[4] = uint64ToLeaf(binary.LittleEndian.Uint64(data[184:192]))

	return merkleize(leaves)
}

// hashPendingPartialWithdrawals computes the hash of pending partial withdrawals.
// Each PendingPartialWithdrawal is 24 bytes (8 + 8 + 8).
func hashPendingPartialWithdrawals(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	const withdrawalSize = 24
	count := len(data) / withdrawalSize
	leaves := make([][32]byte, count)

	for i := 0; i < count; i++ {
		withdrawal := data[i*withdrawalSize : (i+1)*withdrawalSize]
		// 3 uint64 fields: index, amount, withdrawable_epoch
		leaves[i] = hashThreeUint64s(
			binary.LittleEndian.Uint64(withdrawal[0:8]),
			binary.LittleEndian.Uint64(withdrawal[8:16]),
			binary.LittleEndian.Uint64(withdrawal[16:24]),
		)
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashPendingConsolidations computes the hash of pending consolidations.
// Each PendingConsolidation is 16 bytes (8 + 8).
func hashPendingConsolidations(data []byte) [32]byte {
	if len(data) == 0 {
		return mixInLength([32]byte{}, 0)
	}

	const consolidationSize = 16
	count := len(data) / consolidationSize
	leaves := make([][32]byte, count)

	for i := 0; i < count; i++ {
		consolidation := data[i*consolidationSize : (i+1)*consolidationSize]
		// 2 uint64 fields: source_index, target_index
		leaves[i] = hashTwoUint64s(
			binary.LittleEndian.Uint64(consolidation[0:8]),
			binary.LittleEndian.Uint64(consolidation[8:16]),
		)
	}

	root := merkleize(leaves)
	return mixInLength(root, uint64(count))
}

// hashThreeUint64s hashes three uint64 values as a container
func hashThreeUint64s(a, b, c uint64) [32]byte {
	leaves := make([][32]byte, 4)
	leaves[0] = uint64ToLeaf(a)
	leaves[1] = uint64ToLeaf(b)
	leaves[2] = uint64ToLeaf(c)
	return merkleize(leaves)
}

// hashTwoUint64s hashes two uint64 values as a container
func hashTwoUint64s(a, b uint64) [32]byte {
	leaves := make([][32]byte, 2)
	leaves[0] = uint64ToLeaf(a)
	leaves[1] = uint64ToLeaf(b)
	return merkleize(leaves)
}

// merkleize computes the Merkle root of a list of leaves
func merkleize(leaves [][32]byte) [32]byte {
	if len(leaves) == 0 {
		return [32]byte{}
	}
	if len(leaves) == 1 {
		return leaves[0]
	}

	// Pad to power of 2
	n := nextPowerOfTwo(len(leaves))
	padded := make([][32]byte, n)
	copy(padded, leaves)

	// Build tree bottom-up
	for n > 1 {
		for i := 0; i < n/2; i++ {
			padded[i] = hashTwo(padded[i*2], padded[i*2+1])
		}
		n = n / 2
	}

	return padded[0]
}

// hashTwo computes SHA256(left || right)
func hashTwo(left, right [32]byte) [32]byte {
	var combined [64]byte
	copy(combined[:32], left[:])
	copy(combined[32:], right[:])
	return sha256.Sum256(combined[:])
}

// sha256Hash computes SHA256 of data
func sha256Hash(data []byte) [32]byte {
	return sha256.Sum256(data)
}

// mixInLength mixes the length into a root (for SSZ lists)
func mixInLength(root [32]byte, length uint64) [32]byte {
	var lengthLeaf [32]byte
	binary.LittleEndian.PutUint64(lengthLeaf[:8], length)
	return hashTwo(root, lengthLeaf)
}

// uint64ToLeaf converts a uint64 to a 32-byte SSZ leaf
func uint64ToLeaf(val uint64) [32]byte {
	var leaf [32]byte
	binary.LittleEndian.PutUint64(leaf[:8], val)
	return leaf
}

// nextPowerOfTwo returns the smallest power of 2 >= n
func nextPowerOfTwo(n int) int {
	if n <= 1 {
		return 1
	}
	n--
	n |= n >> 1
	n |= n >> 2
	n |= n >> 4
	n |= n >> 8
	n |= n >> 16
	return n + 1
}
