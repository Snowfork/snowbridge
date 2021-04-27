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

type ConsumerWorker struct {
	in <-chan struct{}
}

func NewConsumerWorker(in <-chan struct{}) *ConsumerWorker {
	return &ConsumerWorker{in}
}

func (w *ConsumerWorker) Name() string { return "ConsumerWorker" }

func (w *ConsumerWorker) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		for range w.in {
			continue
		}
		return nil
	})
	return nil
}

type TerminatingWorker struct{}

func (w *TerminatingWorker) Name() string { return "TerminatingWorker" }

func (w *TerminatingWorker) Start(ctx context.Context, eg *errgroup.Group) error {
	eg.Go(func() error {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			return nil
		}
	})
	return nil
}

func testConfig() *workers.WorkerConfig {
	return &workers.WorkerConfig{
		Enabled:      true,
		RestartDelay: 1,
	}
}

func testLogger() (*logrus.Entry, *logtest.Hook) {
	logger, hook := logtest.NewNullLogger()
	logger.SetLevel(logrus.DebugLevel)
	log := logger.WithField("source", "TestWorkerPool")
	return log, hook
}

func TestCanStopPool(t *testing.T) {
	factory := func() (workers.Worker, *workers.WorkerConfig, error) {
		return &TestWorker{}, testConfig(), nil
	}
	pool := workers.WorkerPool{
		factory,
		factory,
		factory,
	}

	log, hook := testLogger()
	ctx, cancel := context.WithCancel(context.Background())

	errCh := make(chan error)
	go func() {
		errCh <- pool.RunWithContext(ctx, nil, log)
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

func TestDetectsDeadlockedWorker(t *testing.T) {
	ch := make(chan struct{})
	// The consumer worker will run until the incoming channel is closed, ignoring
	// context cancellation.
	factoryConsumer := func() (workers.Worker, *workers.WorkerConfig, error) {
		return NewConsumerWorker(ch), testConfig(), nil
	}
	pool := workers.WorkerPool{factoryConsumer}

	log, _ := testLogger()
	ctx, cancel := context.WithCancel(context.Background())

	deadlockSignal := make(chan struct{})
	onDeadlock := func() error {
		close(deadlockSignal)
		return nil
	}

	go func() {
		pool.RunWithContext(ctx, onDeadlock, log)
	}()

	cancel()
	// NOTE: This will take several seconds
	<-deadlockSignal
	// Stop the deadlock so the test can exit
	close(ch)
}

func TestRestartsTerminatedWorker(t *testing.T) {
	factoryTerminating := func() (workers.Worker, *workers.WorkerConfig, error) {
		return &TerminatingWorker{}, testConfig(), nil
	}
	pool := workers.WorkerPool{factoryTerminating}

	log, hook := testLogger()
	ctx, cancel := context.WithCancel(context.Background())

	go func() {
		pool.RunWithContext(ctx, nil, log)
	}()

	<-time.After(2 * time.Second)
	cancel()

	assert.Greater(t, len(hook.AllEntries()), 2)
	assert.Equal(t, hook.AllEntries()[2].Message, "Starting worker")
	assert.Equal(t, hook.AllEntries()[2].Data["restarts"], 1)
}
