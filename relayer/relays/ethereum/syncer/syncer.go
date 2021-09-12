// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package syncer

import (
	"context"
	"errors"
	"math/big"
	"sync"

	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

type latestBlockInfo struct {
	sync.Mutex
	fetchFinalizedDone bool
	height             uint64
}

// Syncer retrieves headers starting at a given initial height up to the latest.
// Headers are sent to the channel `headers` in order. If the initial height is
// old, (finalized) headers in the canonical chain will be forwarded in quick succession
// until we catch up with the unfinalized headers. From that point onwards, headers
// on all forks are forwarded. A header is considered final if it has at least
// `descendantsUntilFinal` descendants.
type Syncer struct {
	descendantsUntilFinal uint64
	headerCache           HeaderCache
	headers               chan *gethTypes.Header
	loader                HeaderLoader
	newHeaders            chan *gethTypes.Header
	oldHeaders            chan *gethTypes.Header
}

func NewSyncer(descendantsUntilFinal uint64, loader HeaderLoader) *Syncer {
	return &Syncer{
		descendantsUntilFinal: descendantsUntilFinal,
		headerCache:           *NewHeaderCache(descendantsUntilFinal + 1),
		headers:               nil,
		loader:                loader,
		newHeaders:            nil,
		oldHeaders:            nil,
	}
}

func (s *Syncer) StartSync(
	ctx context.Context,
	eg *errgroup.Group,
	initBlockHeight uint64,
) (<-chan *gethTypes.Header, error) {
	lbi := &latestBlockInfo{
		fetchFinalizedDone: false,
		height:             0,
	}

	s.headers = make(chan *gethTypes.Header, 5)
	s.newHeaders = make(chan *gethTypes.Header)
	s.oldHeaders = make(chan *gethTypes.Header)

	eg.Go(func() error {
		defer close(s.newHeaders)
		err := s.pollNewHeaders(ctx, lbi)
		if err != nil {
			log.WithField("reason", err).Info("Shutting down new headers poller")
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	lbi.Lock()
	defer lbi.Unlock()
	latestHeader, err := s.loader.HeaderByNumber(ctx, nil)
	if err != nil {
		log.WithError(err).Error("Failed to retrieve latest header")
		return nil, err
	}
	if latestHeader.Number.Uint64() > lbi.height {
		lbi.height = latestHeader.Number.Uint64()
	}

	eg.Go(func() error {
		defer close(s.oldHeaders)
		err := s.fetchFinalizedHeaders(ctx, initBlockHeight, lbi)
		if err != nil {
			log.WithField("reason", err).Info("Shutting down finalized headers poller")
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}
		return nil
	})

	eg.Go(func() error {
		defer close(s.headers)
		for header := range s.oldHeaders {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case s.headers <- header:
			}
		}
		for header := range s.newHeaders {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case s.headers <- header:
			}
		}
		return nil
	})

	return s.headers, nil
}

func (s *Syncer) fetchFinalizedHeaders(ctx context.Context, initBlockHeight uint64, lbi *latestBlockInfo) error {
	syncedUpUntil := initBlockHeight

	for {
		lbi.Lock()
		latestFinalizedHeight := saturatingSub(lbi.height, s.descendantsUntilFinal)
		if syncedUpUntil >= latestFinalizedHeight {
			// Signals to pollNewHeaders that new headers can be forwarded now
			lbi.fetchFinalizedDone = true
			lbi.Unlock()

			log.WithField("blockNumber", syncedUpUntil).Debug("Done retrieving finalized headers")

			break
		}
		lbi.Unlock()

		header, err := s.loader.HeaderByNumber(ctx, new(big.Int).SetUint64(syncedUpUntil+1))
		if err != nil {
			log.WithField(
				"blockNumber", syncedUpUntil+1,
			).WithError(err).Error("Failed to retrieve finalized header")
			return err
		}

		log.WithFields(logrus.Fields{
			"blockHash":   header.Hash().Hex(),
			"blockNumber": syncedUpUntil + 1,
		}).Debug("Retrieved finalized header")

		select {
		case <-ctx.Done():
			return ctx.Err()
		case s.oldHeaders <- header:
		}

		syncedUpUntil++
	}

	return nil
}

func (s *Syncer) pollNewHeaders(ctx context.Context, lbi *latestBlockInfo) error {
	headers := make(chan *gethTypes.Header)

	sub, err := s.loader.SubscribeNewHead(ctx, headers)
	if err != nil {
		log.WithError(err).Error("Failed to subscribe to new headers")
		return err
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			return err
		case header := <-headers:
			s.headerCache.Insert(header)
			lbi.Lock()
			lbi.height = header.Number.Uint64()

			log.WithFields(logrus.Fields{
				"blockHash":   header.Hash().Hex(),
				"blockNumber": lbi.height,
			}).Debug("Witnessed new header")

			if lbi.fetchFinalizedDone {
				err = s.forwardAncestry(ctx, header.Hash(), saturatingSub(lbi.height, s.descendantsUntilFinal))
				if err != nil {
					log.WithFields(logrus.Fields{
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
		header, err := s.loader.HeaderByHash(ctx, hash)
		if err != nil {
			return err
		}

		// If a header is too old, it cannot be inserted. We can assume it's already been forwarded
		if !s.headerCache.Insert(header) {
			return nil
		}
		item, _ = s.headerCache.Get(hash)
	}

	if item.Forwarded {
		return nil
	}

	if item.Header.Number.Uint64() > oldestHeight {
		err := s.forwardAncestry(ctx, item.Header.ParentHash, oldestHeight)
		if err != nil {
			return err
		}
	}

	select {
	case <-ctx.Done():
		return ctx.Err()
	case s.newHeaders <- item.Header:
	}

	item.Forwarded = true
	return nil
}

// Subtraction but returns 0 when r > l
func saturatingSub(l uint64, r uint64) uint64 {
	if r > l {
		return 0
	}
	return l - r
}
