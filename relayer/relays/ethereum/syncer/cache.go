// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package syncer

import (
	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
)

type HeaderCacheItem struct {
	Header    *gethTypes.Header
	Forwarded bool
}

// This is used to store the latest headers as they are published. Up to
// `numHeightsToTrack` heights are stored. Once this number is reached, an old
// height is pruned each time a new height is added. The current stored height
// range is given by [minHeight, maxHeight].
type HeaderCache struct {
	headers           map[string]*HeaderCacheItem
	hashesByHeight    map[uint64][]string
	maxHeight         uint64
	minHeight         uint64
	numHeightsToTrack uint64
}

func NewHeaderCache(numHeightsToTrack uint64) *HeaderCache {
	return &HeaderCache{
		headers:           make(map[string]*HeaderCacheItem, numHeightsToTrack),
		hashesByHeight:    make(map[uint64][]string, numHeightsToTrack),
		maxHeight:         0,
		minHeight:         ^uint64(0),
		numHeightsToTrack: numHeightsToTrack,
	}
}

// Returns true if insertion was successful. Insertion will only fail
// if a header is too old, i.e. we've seen at least `numHeightsToTrack`
// newer heights
func (hc *HeaderCache) Insert(header *gethTypes.Header) bool {
	hash := header.Hash().Hex()
	_, exists := hc.headers[hash]
	if exists {
		return true
	}

	// Don't track headers older than numHeightsToTrack. This means
	// the range [minHeight, maxHeight] is always moving forward
	height := header.Number.Uint64()
	if hc.maxHeight >= hc.numHeightsToTrack && height <= hc.maxHeight-hc.numHeightsToTrack {
		return false
	}

	hc.headers[hash] = &HeaderCacheItem{
		Header:    header,
		Forwarded: false,
	}

	hashesAtHeight, heightExists := hc.hashesByHeight[height]
	if heightExists {
		hc.hashesByHeight[height] = append(hashesAtHeight, hash)
	} else {
		hc.hashesByHeight[height] = []string{hash}
		if height < hc.minHeight {
			hc.minHeight = height
		} else if height > hc.maxHeight {
			hc.maxHeight = height
		}
	}

	if hc.maxHeight >= hc.numHeightsToTrack {
		hc.pruneUpTo(hc.maxHeight - hc.numHeightsToTrack + 1)
	}
	return true
}

func (hc *HeaderCache) pruneUpTo(minHeightToKeep uint64) {
	for hc.minHeight < minHeightToKeep {
		hashesToRemove := hc.hashesByHeight[hc.minHeight]
		delete(hc.hashesByHeight, hc.minHeight)
		hc.minHeight++
		for _, hashToRemove := range hashesToRemove {
			delete(hc.headers, hashToRemove)
		}
	}
}

func (hc *HeaderCache) Get(hash gethCommon.Hash) (*HeaderCacheItem, bool) {
	hashHex := hash.Hex()
	item, exists := hc.headers[hashHex]
	if exists {
		return item, true
	}
	return nil, false
}
