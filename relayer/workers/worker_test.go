// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package workers_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/sirupsen/logrus"
	logtest "github.com/sirupsen/logrus/hooks/test"

	"github.com/snowfork/polkadot-ethereum/relayer/workers"
	"github.com/stretchr/testify/assert"
	"golang.org/x/sync/errgroup"
)

type TestWorker struct{}

func (w *TestWorker) Name() string { return "TestWorker" }

func (w *TestWorker) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		<-ctx.Done()
		return ctx.Err()
	})
	return nil
}

func TestCanStopPool(t *testing.T) {
	factory := func() (workers.Worker, error) { return &TestWorker{}, nil }
	pool := workers.WorkerPool{
		factory,
		factory,
		factory,
	}

	logger, hook := logtest.NewNullLogger()
	logger.SetLevel(logrus.DebugLevel)

	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error)
	go func() {
		errCh <- pool.RunWithContext(ctx, logger.WithField("source", "TestWorkerPool"))
	}()

	<-time.After(100 * time.Millisecond)
	cancel()
	terminalErr := <-errCh

	assert.Equal(t, errors.New("context canceled"), terminalErr)
	// 3 starts followed by 3 stops should have been logged
	assert.Equal(t, 6, len(hook.AllEntries()))
	assert.Equal(t, hook.AllEntries()[2].Message, "Starting worker")
	assert.Equal(t, hook.LastEntry().Message, "Worker terminated")
}
