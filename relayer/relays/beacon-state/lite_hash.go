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

// hashHistoricalRoots computes the hash tree root of the HistoricalRoots list.
// Each element is a 32-byte root. Limit is 2^24 = 16777216.
func hashHistoricalRoots(data []byte) [32]byte {
	const limit uint64 = 16777216 // 2^24
	// Each element is 32 bytes, so chunk limit = limit
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
	}

	count := len(data) / 32
	leaves := make([][32]byte, count)
	for i := 0; i < count; i++ {
		copy(leaves[i][:], data[i*32:(i+1)*32])
	}

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashEth1DataVotes computes the hash tree root of the Eth1DataVotes list.
// Each Eth1Data is 72 bytes. Limit is 2048 (EPOCHS_PER_ETH1_VOTING_PERIOD * SLOTS_PER_EPOCH).
func hashEth1DataVotes(data []byte) [32]byte {
	const limit uint64 = 2048
	// Each Eth1Data hashes to one 32-byte leaf
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
	}

	const eth1DataSize = 72
	count := len(data) / eth1DataSize
	leaves := make([][32]byte, count)
	for i := 0; i < count; i++ {
		element := data[i*eth1DataSize : (i+1)*eth1DataSize]
		leaves[i] = hashEth1Data(element)
	}

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashEth1Data computes the hash of a single Eth1Data (72 bytes)
func hashEth1Data(data []byte) [32]byte {
	if len(data) < 72 {
		return [32]byte{}
	}
	// Eth1Data: deposit_root (32) + deposit_count (8) + block_hash (32)
	leaves := make([][32]byte, 4) // 3 fields, padded to 4 for merkleization
	copy(leaves[0][:], data[0:32])
	leaves[1] = uint64ToLeaf(binary.LittleEndian.Uint64(data[32:40]))
	copy(leaves[2][:], data[40:72])
	return merkleize(leaves)
}

// hashValidators computes the hash tree root of the validators list.
// Each validator is 121 bytes. We hash them in a streaming fashion.
// SSZ lists require merkleization to a depth based on the limit, not actual count.
// Validator list limit is 2^40 = 1099511627776.
func hashValidators(data []byte) [32]byte {
	const validatorSize = 121
	const validatorLimit = 1099511627776 // 2^40

	if len(data) == 0 {
		// Empty list: merkle root at depth 40, mixed with length 0
		root := merkleizeWithLimit(nil, validatorLimit)
		return mixInLength(root, 0)
	}

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

	root := merkleizeWithLimit(leaves, validatorLimit)
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

	// Field 0: pubkey (48 bytes -> 2 chunks, merkleized)
	// SSZ chunks bytes into 32-byte pieces and merkleizes
	var pubkeyC1, pubkeyC2 [32]byte
	copy(pubkeyC1[:], data[0:32])
	copy(pubkeyC2[:], data[32:48]) // bytes 32-47, rest is zeros
	leaves[0] = hashTwo(pubkeyC1, pubkeyC2)

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
// SSZ lists require merkleization to a depth based on the limit.
// Balances list limit is 2^40 = 1099511627776.
// Chunk count for uint64 list: (limit * 8 + 31) / 32 = limit / 4 (when limit is power of 2)
func hashBalances(data []byte) [32]byte {
	const balancesLimit uint64 = 1099511627776 // 2^40
	// For uint64 elements (8 bytes), chunk limit = (limit * 8 + 31) / 32
	const chunkLimit uint64 = (balancesLimit*8 + 31) / 32 // = 2^38

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
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
// This is a byte list (one byte per validator). Limit is 2^40 = 1099511627776.
func hashParticipation(data []byte) [32]byte {
	const limit uint64 = 1099511627776 // 2^40
	// Each byte is 1 byte, 32 bytes per chunk, so chunk limit = (limit + 31) / 32
	const chunkLimit = (limit + 31) / 32

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(len(data)))
}

// hashInactivityScores computes the hash tree root of inactivity scores.
// List of uint64 values. Limit is 2^40 = 1099511627776.
func hashInactivityScores(data []byte) [32]byte {
	const limit uint64 = 1099511627776 // 2^40
	// For uint64 elements (8 bytes), chunk limit = (limit * 8 + 31) / 32 = 2^38
	const chunkLimit = (limit*8 + 31) / 32

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashHistoricalSummaries computes the hash tree root of historical summaries.
// Each summary is 64 bytes (two 32-byte roots). Limit is 2^24 = 16777216.
func hashHistoricalSummaries(data []byte) [32]byte {
	const limit uint64 = 16777216 // 2^24
	// Each HistoricalSummary hashes to one 32-byte leaf
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashProposerLookahead computes the hash tree root of the proposer lookahead.
// Fixed-size vector of 64 uint64s (512 bytes).
func hashProposerLookahead(data []byte) [32]byte {
	const count = 64
	// 64 uint64s = 16 chunks of 4 uint64s each
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

	// Fixed-size vector, no mix-in length
	return merkleize(leaves)
}

// hashPendingDeposits computes the hash tree root of pending deposits.
// Each PendingDeposit is 192 bytes (48 + 32 + 8 + 96 + 8). Limit is 2^27 = 134217728.
func hashPendingDeposits(data []byte) [32]byte {
	const limit uint64 = 134217728 // 2^27
	// Each PendingDeposit hashes to one 32-byte leaf
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
	}

	const depositSize = 192
	count := len(data) / depositSize
	leaves := make([][32]byte, count)

	for i := 0; i < count; i++ {
		deposit := data[i*depositSize : (i+1)*depositSize]
		leaves[i] = hashPendingDeposit(deposit)
	}

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashPendingDeposit hashes a single PendingDeposit
// PendingDeposit has 5 fields, each producing a single 32-byte field root:
// - pubkey (48 bytes) -> merkleized to 1 root
// - withdrawal_credentials (32 bytes) -> 1 root
// - amount (8 bytes) -> 1 root
// - signature (96 bytes) -> merkleized to 1 root
// - index (8 bytes) -> 1 root
// These 5 field roots are then merkleized (padded to 8 leaves)
func hashPendingDeposit(data []byte) [32]byte {
	if len(data) < 192 {
		return [32]byte{}
	}

	leaves := make([][32]byte, 8)

	// Field 0: pubkey (48 bytes) -> merkleize 2 chunks to get root
	var pubkeyC1, pubkeyC2 [32]byte
	copy(pubkeyC1[:], data[0:32])
	copy(pubkeyC2[:], data[32:48]) // bytes 32-47, rest is zeros
	leaves[0] = hashTwo(pubkeyC1, pubkeyC2)

	// Field 1: withdrawal_credentials (32 bytes) -> already a root
	copy(leaves[1][:], data[48:80])

	// Field 2: amount (8 bytes)
	leaves[2] = uint64ToLeaf(binary.LittleEndian.Uint64(data[80:88]))

	// Field 3: signature (96 bytes) -> merkleize 3 chunks (padded to 4) to get root
	var sigC1, sigC2, sigC3, sigC4 [32]byte
	copy(sigC1[:], data[88:120])
	copy(sigC2[:], data[120:152])
	copy(sigC3[:], data[152:184])
	// sigC4 is zeros (padding to power of 2)
	leaves[3] = hashTwo(hashTwo(sigC1, sigC2), hashTwo(sigC3, sigC4))

	// Field 4: index (8 bytes)
	leaves[4] = uint64ToLeaf(binary.LittleEndian.Uint64(data[184:192]))

	// leaves[5-7] are zeros (padding to 8)

	return merkleize(leaves)
}

// hashPendingPartialWithdrawals computes the hash of pending partial withdrawals.
// Each PendingPartialWithdrawal is 24 bytes (8 + 8 + 8). Limit is 2^27 = 134217728.
func hashPendingPartialWithdrawals(data []byte) [32]byte {
	const limit uint64 = 134217728 // 2^27
	// Each withdrawal hashes to one 32-byte leaf
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
	return mixInLength(root, uint64(count))
}

// hashPendingConsolidations computes the hash of pending consolidations.
// Each PendingConsolidation is 16 bytes (8 + 8). Limit is 2^18 = 262144.
func hashPendingConsolidations(data []byte) [32]byte {
	const limit uint64 = 262144 // 2^18
	// Each consolidation hashes to one 32-byte leaf
	const chunkLimit = limit

	if len(data) == 0 {
		root := merkleizeWithLimit(nil, chunkLimit)
		return mixInLength(root, 0)
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

	root := merkleizeWithLimit(leaves, chunkLimit)
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

// merkleizeWithLimit computes the Merkle root for an SSZ list with a specific limit.
// The tree depth is based on the limit, not the actual count of elements.
// This matches fastssz's MerkleizeWithMixin behavior.
func merkleizeWithLimit(leaves [][32]byte, limit uint64) [32]byte {
	// Calculate the depth needed for the limit
	depth := getDepth(limit)

	if len(leaves) == 0 {
		// Return zero hash at the appropriate depth
		return getZeroHash(depth)
	}

	if len(leaves) == 1 && depth == 0 {
		return leaves[0]
	}

	// Build tree from bottom up
	layer := make([][32]byte, len(leaves))
	copy(layer, leaves)

	for d := uint8(0); d < depth; d++ {
		// Calculate size of next layer (round up)
		nextSize := (len(layer) + 1) / 2

		// If this is the last layer before reaching the target depth,
		// we might need to combine with zero hashes
		newLayer := make([][32]byte, nextSize)

		for i := 0; i < len(layer); i += 2 {
			left := layer[i]
			var right [32]byte
			if i+1 < len(layer) {
				right = layer[i+1]
			} else {
				right = getZeroHash(d)
			}
			newLayer[i/2] = hashTwo(left, right)
		}

		layer = newLayer
	}

	// If we still have more than one element, continue until we have one
	for len(layer) > 1 {
		newLayer := make([][32]byte, (len(layer)+1)/2)
		for i := 0; i < len(layer); i += 2 {
			left := layer[i]
			var right [32]byte
			if i+1 < len(layer) {
				right = layer[i+1]
			}
			newLayer[i/2] = hashTwo(left, right)
		}
		layer = newLayer
	}

	return layer[0]
}

// getDepth returns the depth of a merkle tree for a given limit
func getDepth(limit uint64) uint8 {
	if limit <= 1 {
		return 0
	}
	depth := uint8(0)
	for (uint64(1) << depth) < limit {
		depth++
	}
	return depth
}

// zeroHashes contains precomputed zero hashes at each depth
var zeroHashes [][32]byte

func init() {
	// Precompute zero hashes up to depth 64 (more than enough for any SSZ type)
	zeroHashes = make([][32]byte, 65)
	zeroHashes[0] = [32]byte{}
	for i := 1; i < 65; i++ {
		zeroHashes[i] = hashTwo(zeroHashes[i-1], zeroHashes[i-1])
	}
}

// getZeroHash returns the zero hash at a given depth
func getZeroHash(depth uint8) [32]byte {
	if int(depth) < len(zeroHashes) {
		return zeroHashes[depth]
	}
	// Should not happen, but compute if needed
	hash := zeroHashes[len(zeroHashes)-1]
	for i := len(zeroHashes) - 1; i < int(depth); i++ {
		hash = hashTwo(hash, hash)
	}
	return hash
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
