// Copyright 2020 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package syncer

import (
	"context"
	"errors"
	"math/big"

	gethCommon "github.com/ethereum/go-ethereum/common"
	gethTypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"

	log "github.com/sirupsen/logrus"
)

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
}

func NewSyncer(
	descendantsUntilFinal uint64,
	loader HeaderLoader,
) *Syncer {
	return &Syncer{
		descendantsUntilFinal: descendantsUntilFinal,
		headerCache:           *NewHeaderCache(descendantsUntilFinal + 1),
		headers:               nil,
		loader:                loader,
	}
}

func (s *Syncer) StartSync(
	ctx context.Context,
	eg *errgroup.Group,
	initBlockHeight uint64,
) (chan *gethTypes.Header, error) {
	var height uint64

	s.headers = make(chan *gethTypes.Header, 5)

	latestHeader, err := s.loader.HeaderByNumber(ctx, nil)
	if err != nil {
		log.WithError(err).Error("Failed to retrieve latest header")
		return nil, err
	}
	if latestHeader.Number.Uint64() > height {
		height = latestHeader.Number.Uint64()
	}

	eg.Go(func() error {
		defer close(s.headers)

		err = s.fetchNewHeaders(ctx)
		if err != nil {
			log.WithField("reason", err).Info("Shutting down header syncer")
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}

		err := s.fetchFinalizedHeaders(ctx, initBlockHeight, height)
		if err != nil {
			log.WithField("reason", err).Info("Shutting down header syncer")
			if errors.Is(err, context.Canceled) {
				return nil
			}
			return err
		}


		return nil
	})

	return s.headers, nil
}

func (s *Syncer) fetchFinalizedHeaders(ctx context.Context, initBlockHeight uint64, height uint64) error {
	syncedUpUntil := initBlockHeight

	for {
		latestFinalizedHeight := saturatingSub(height, s.descendantsUntilFinal)
		if syncedUpUntil >= latestFinalizedHeight {
			log.WithField("blockNumber", syncedUpUntil).Debug("Done retrieving finalized headers")
			break
		}

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
		case s.headers <- header:
		}

		syncedUpUntil++
	}

	return nil
}

func (s *Syncer) fetchNewHeaders(ctx context.Context) error {
	headersIn := make(chan *gethTypes.Header)

	sub, err := s.loader.SubscribeNewHead(ctx, headersIn)
	if err != nil {
		log.WithError(err).Error("Failed to subscribe to new headers")
		return err
	}
	defer sub.Unsubscribe()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case err := <-sub.Err():
			log.WithError(err).Error("Header subscription failed")
			return err
		case header, ok := <-headersIn:
			if !ok {
				return nil
			}
			s.headerCache.Insert(header)
			height := header.Number.Uint64()

			log.WithFields(logrus.Fields{
				"blockHash":   header.Hash().Hex(),
				"blockNumber": height,
			}).Debug("Witnessed new header")

			err = s.forwardAncestry(ctx, header.Hash(), saturatingSub(height, s.descendantsUntilFinal))
			if err != nil {
				return err
			}
		}
	}
}

func (s *Syncer) forwardAncestry(ctx context.Context, hash gethCommon.Hash, oldestHeight uint64) error {
	log.WithFields(logrus.Fields{
		"blockHash":   hash,
		"oldestHeight": oldestHeight,
	}).Info("Forwarding header and its ancestors")

	item, exists := s.headerCache.Get(hash)
	if !exists {
		header, err := s.loader.HeaderByHash(ctx, hash)
		if err != nil {
			log.WithFields(logrus.Fields{
				"blockHash":   hash,
				"error": err,
			}).Error("Failed to fetch block header")
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
	case s.headers <- item.Header:
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
