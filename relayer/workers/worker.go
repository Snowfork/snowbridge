// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package workers

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"golang.org/x/sync/errgroup"
)

type Worker interface {
	Name() string

	SetUp() error
	TearDown()

	Start(ctx context.Context, eg *errgroup.Group) error
}

type WorkerPool struct {
	workers []Worker
}

func NewWorkerPool(workers []Worker) *WorkerPool {
	return &WorkerPool{
		workers,
	}
}

func (wp *WorkerPool) runWorker(ctx context.Context, worker Worker) error {
	// Ensure we always clean up after ourselves
	defer worker.TearDown()

	err := worker.SetUp()
	if err != nil {
		return err
	}

	childEg, childCtx := errgroup.WithContext(ctx)
	err = worker.Start(childCtx, childEg)
	if err != nil {
		return err
	}

	return childEg.Wait()
}

func (wp *WorkerPool) Run() error {
	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)

	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-notify:
			// TODO: add logging back in
			//log.WithField("signal", sig.String()).Info("Received signal")
			cancel()
		}

		return nil
	})

	for _, w := range wp.workers {
		worker := w
		eg.Go(func() error {
			for {
				// TODO: log starting worker
				err := wp.runWorker(ctx, worker)
				// TODO: log ending worker
				if err != nil {
					// TODO: retry with backoff up to X retries
					return err
				}
			}
		})
	}

	return eg.Wait()
}
