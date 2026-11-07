package executionV3

import (
	"context"
	"sort"

	log "github.com/sirupsen/logrus"
	"github.com/snowfork/snowbridge/relayer/chain/ethereum"
)

// resolveNonceBlocks maps each nonce to the finalized block that emitted it.
// Cached blocks are reused; the rest are found with ethereum.NonceBlock, each
// search starting from the previous nonce's block. The cache keeps only the
// nonces asked for, so delivered nonces drop out on the next scan.
func (r *Relay) resolveNonceBlocks(
	ctx context.Context,
	latestBlock uint64,
	nonces []uint64,
	cache map[uint64]uint64,
	nonceAt func(ctx context.Context, block uint64) (uint64, error),
) (map[uint64]uint64, error) {
	sorted := append([]uint64(nil), nonces...)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i] < sorted[j] })

	r.nonceBlocksMu.Lock()
	blocks := make(map[uint64]uint64, len(sorted))
	for _, n := range sorted {
		if b, ok := cache[n]; ok {
			blocks[n] = b
		}
	}
	r.nonceBlocksMu.Unlock()

	var from uint64
	for _, n := range sorted {
		if b, ok := blocks[n]; ok {
			from = b
			continue
		}
		b, err := ethereum.NonceBlock(ctx, from, latestBlock, n, nonceAt)
		if err != nil {
			return nil, err
		}
		blocks[n] = b
		from = b
	}

	r.nonceBlocksMu.Lock()
	for n := range cache {
		if _, ok := blocks[n]; !ok {
			delete(cache, n)
		}
	}
	for n, b := range blocks {
		cache[n] = b
	}
	r.nonceBlocksMu.Unlock()
	return blocks, nil
}

// forgetMissingNonces drops cached blocks whose log did not contain the nonce,
// so the next scan searches for them again.
func (r *Relay) forgetMissingNonces(cache, blocks map[uint64]uint64, found map[uint64]struct{}) {
	r.nonceBlocksMu.Lock()
	defer r.nonceBlocksMu.Unlock()
	for n, b := range blocks {
		if _, ok := found[n]; !ok {
			log.WithFields(log.Fields{"nonce": n, "block": b}).Warn("[instant-eth] nonce event not found in its block")
			delete(cache, n)
		}
	}
}

func distinctBlocks(blocks map[uint64]uint64) []uint64 {
	seen := make(map[uint64]struct{}, len(blocks))
	var out []uint64
	for _, b := range blocks {
		if _, ok := seen[b]; !ok {
			seen[b] = struct{}{}
			out = append(out, b)
		}
	}
	sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })
	return out
}
