// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package ethereum

import (
	"context"
	"math/big"
	"sync"

	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type latestBlockInfo struct {
	sync.Mutex
	fetchFinalizedDone bool
	height             uint64
}

type headerCacheItem struct {
	header    *gethTypes.Header
	forwarded bool
}

type headerCache struct {
	headers           map[string]headerCacheItem
	hashesByHeight    map[uint64][]string
	maxHeight         uint64
	minHeight         uint64
	numHeightsToTrack uint64
}

func (hc *headerCache) Insert(header *gethTypes.Header) bool {
	hash := header.Hash().Hex()
	_, exists := hc.headers[hash]
	if exists {
		return true
	}

	// Don't track headers older than numHeightsToTrack. This means
	// the range [minHeight, maxHeight] is always moving forward
	height := header.Number.Uint64()
	if height < hc.maxHeight-hc.numHeightsToTrack {
		return false
	}

	hc.headers[hash] = headerCacheItem{
		header:    header,
		forwarded: false,
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

	hc.pruneUpTo(hc.maxHeight - hc.numHeightsToTrack)
	return true
}

func (hc *headerCache) pruneUpTo(minHeightToKeep uint64) {
	for h := hc.minHeight; h < minHeightToKeep; h++ {
		hashesToRemove := hc.hashesByHeight[h]
		delete(hc.hashesByHeight, h)
		for _, hashToRemove := range hashesToRemove {
			delete(hc.headers, hashToRemove)
		}
	}
	hc.minHeight = minHeightToKeep
}

func (hc *headerCache) Get(hash gethCommon.Hash) (*headerCacheItem, bool) {
	hashHex := hash.Hex()
	item, exists := hc.headers[hashHex]
	if exists {
		return &item, true
	}
	return nil, false
}

type Syncer struct {
	conn                  *Connection
	descendantsUntilFinal uint64
	headerCache           headerCache
	headers               chan<- *gethTypes.Header
	log                   *logrus.Entry
}

func NewSyncer(conn *Connection, descendantsUntilFinal uint64, headers chan<- *gethTypes.Header, log *logrus.Entry) *Syncer {
	return &Syncer{
		conn:                  conn,
		descendantsUntilFinal: descendantsUntilFinal,
		headerCache: headerCache{
			headers:           make(map[string]headerCacheItem, descendantsUntilFinal),
			hashesByHeight:    make(map[uint64][]string, descendantsUntilFinal),
			maxHeight:         0,
			minHeight:         ^uint64(0),
			numHeightsToTrack: descendantsUntilFinal,
		},
		headers: headers,
		log:     log,
	}
}

func (s *Syncer) StartSync(ctx context.Context, eg *errgroup.Group, initBlockHeight uint64) error {
	latestHeader, err := s.conn.client.HeaderByNumber(ctx, nil)
	if err != nil {
		s.log.WithError(err).Error("Failed to retrieve latest header")
		return err
	}

	lbi := &latestBlockInfo{
		fetchFinalizedDone: false,
		height:             latestHeader.Number.Uint64(),
	}

	eg.Go(func() error {
		return s.fetchFinalizedHeaders(ctx, initBlockHeight, lbi)
	})
	eg.Go(func() error {
		return s.pollNewHeaders(ctx, lbi)
	})

	return nil
}

func (s *Syncer) fetchFinalizedHeaders(ctx context.Context, initBlockHeight uint64, lbi *latestBlockInfo) error {
	syncedUpUntil := initBlockHeight

	for {
		lbi.Lock()
		latestFinalizedHeight := lbi.height - s.descendantsUntilFinal
		if syncedUpUntil >= latestFinalizedHeight {
			// Signals to pollNewHeaders that new headers can be forwarded now
			lbi.fetchFinalizedDone = true
			lbi.Unlock()

			s.log.WithField("blockNumber", syncedUpUntil).Debug("Done retrieving finalized headers")

			break
		}
		lbi.Unlock()

		header, err := s.conn.client.HeaderByNumber(ctx, new(big.Int).SetUint64(syncedUpUntil+1))
		if err != nil {
			s.log.WithField(
				"blockNumber", syncedUpUntil+1,
			).WithError(err).Error("Failed to retrieve finalized header")
			// Only option is to retry
			continue
		}

		s.log.WithFields(logrus.Fields{
			"blockHash":   header.Hash().Hex(),
			"blockNumber": syncedUpUntil + 1,
		}).Debug("Retrieved finalized header")

		s.headers <- header
		syncedUpUntil++
	}

	return nil
}

func (s *Syncer) pollNewHeaders(ctx context.Context, lbi *latestBlockInfo) error {
	headers := make(chan *gethTypes.Header)
	var headersSubscriptionErr <-chan error

	subscription, err := s.conn.client.SubscribeNewHead(ctx, headers)
	if err != nil {
		s.log.WithError(err).Error("Failed to subscribe to new headers")
		return err
	}
	headersSubscriptionErr = subscription.Err()

	for {
		select {
		case <-ctx.Done():
			close(s.headers)
			return ctx.Err()
		case err := <-headersSubscriptionErr:
			close(s.headers)
			return err
		case header := <-headers:
			s.headerCache.Insert(header)
			lbi.Lock()
			lbi.height = header.Number.Uint64()

			s.log.WithFields(logrus.Fields{
				"blockHash":   header.Hash().Hex(),
				"blockNumber": lbi.height,
			}).Debug("Witnessed new header")

			if lbi.fetchFinalizedDone {
				err = s.forwardAncestry(ctx, header.Hash(), lbi.height-s.descendantsUntilFinal)
				if err != nil {
					s.log.WithFields(logrus.Fields{
						"blockHash":   header.Hash().Hex(),
						"blockNumber": lbi.height,
					}).WithError(err).Error("Failed to forward header and its ancestors")
				}
			}
			lbi.Unlock()
		}
	}
}

func (s *Syncer) forwardAncestry(ctx context.Context, hash gethCommon.Hash, oldestHeight uint64) error {
	item, exists := s.headerCache.Get(hash)
	if !exists {
		header, err := s.conn.client.HeaderByHash(ctx, hash)
		if err != nil {
			return err
		}

		// If a header is too old, it cannot be inserted. We can assume it's already been forwarded
		if !s.headerCache.Insert(header) {
			return nil
		}
		item, _ = s.headerCache.Get(hash)
	}

	if item.forwarded {
		return nil
	}

	if item.header.Number.Uint64() > oldestHeight {
		err := s.forwardAncestry(ctx, item.header.ParentHash, oldestHeight)
		if err != nil {
			return err
		}
	}

	s.headers <- item.header
	item.forwarded = true
	return nil
}
