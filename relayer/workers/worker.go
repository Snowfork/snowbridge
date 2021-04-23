// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package workers

import (
	"context"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/sirupsen/logrus"
	"golang.org/x/sync/errgroup"
)

type Worker interface {
	Name() string
	Start(ctx context.Context, eg *errgroup.Group) error
}

type WorkerFactory func() (Worker, error)

type WorkerPool []WorkerFactory

func (wp WorkerPool) runWorker(ctx context.Context, worker Worker) error {
	childEg, childCtx := errgroup.WithContext(ctx)
	err := worker.Start(childCtx, childEg)
	if err != nil {
		return err
	}

	// We wait for this worker to finish in an indepedent goroutine. This
	// allows us to detect when a worker is deadlocked, i.e. all its
	// goroutines are not terminating when childCtx.Done() is signaled.
	// If a deadlock occurs, we have to kill the process to clean up
	// the worker.
	notifyWaitDone := make(chan struct{})
	var terminalErr error = nil

	go func() {
		terminalErr = childEg.Wait()
		close(notifyWaitDone)
	}()

	select {
	case <-notifyWaitDone:
		return terminalErr
	case <-childCtx.Done():
		// Goroutines are either shutting down or deadlocked.
		// Give them a few seconds...
		select {
		case <-time.After(3 * time.Second):
			break
		case _, stillWaiting := <-notifyWaitDone:
			if !stillWaiting {
				// All goroutines have ended
				return terminalErr
			}
		}

		wp.getLogger().WithField(
			"worker",
			worker.Name(),
		).Error("The worker's goroutines are deadlocked. Please fix")

		relayProc, _ := os.FindProcess(os.Getpid())
		relayProc.Kill()
		return nil
	}
}

func (wp WorkerPool) Run() error {
	log := wp.getLogger()

	ctx, cancel := context.WithCancel(context.Background())
	eg, ctx := errgroup.WithContext(ctx)

	// Ensure clean termination upon SIGINT, SIGTERM
	eg.Go(func() error {
		notify := make(chan os.Signal, 1)
		signal.Notify(notify, syscall.SIGINT, syscall.SIGTERM)

		select {
		case <-ctx.Done():
			return ctx.Err()
		case sig := <-notify:
			log.WithField("signal", sig.String()).Info("Received signal")
			cancel()
		}

		return nil
	})

	for _, f := range wp {
		factory := f

		eg.Go(func() error {
			for {
				worker, err := factory()
				if err != nil {
					// It is unrecoverable if we cannot construct one of our workers
					return err
				}

				log.WithField("worker", worker.Name()).Debug("Starting worker")
				err = wp.runWorker(ctx, worker)
				log.WithField("worker", worker.Name()).Debug("Worker terminated")

				select {
				case <-ctx.Done():
					return ctx.Err()
				default:
					// TODO: instead retry with backoff up to X retries
					return err
				}
			}
		})
	}

	return eg.Wait()
}

func (wp WorkerPool) getLogger() *logrus.Entry {
	return logrus.WithField("source", "WorkerPool")
}
