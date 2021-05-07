// Copyright 2021 Snowfork
// SPDX-License-Identifier: LGPL-3.0-only

package workers

import (
	"context"
	"errors"
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

type WorkerFactory func() (Worker, *WorkerConfig, error)

type WorkerPool []WorkerFactory

var WorkerDeadlocked = errors.New("Worker deadlocked")

type DeadlockHandler func() error

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
	notifyWaitDone := make(chan error)

	go func() {
		notifyWaitDone <- childEg.Wait()
		close(notifyWaitDone)
	}()

	select {
	case err := <-notifyWaitDone:
		return err
	case <-childCtx.Done():
		// Goroutines are either shutting down or deadlocked.
		// Give them a few seconds...
		select {
		case <-time.After(10 * time.Second):
			break
		case err := <-notifyWaitDone:
			// All goroutines have ended
			return err
		}

		return WorkerDeadlocked
	}
}

func (wp WorkerPool) Run() error {
	return wp.run(context.Background(), nil, wp.defaultLogger())
}

func (wp WorkerPool) RunWithContext(ctx context.Context, onDeadlock DeadlockHandler, log *logrus.Entry) error {
	return wp.run(ctx, onDeadlock, log)
}

func (wp WorkerPool) run(ctx context.Context, onDeadlock DeadlockHandler, log *logrus.Entry) error {
	ctx, cancel := context.WithCancel(ctx)
	eg, ctx := errgroup.WithContext(ctx)

	if onDeadlock == nil {
		onDeadlock = func() error {
			cancel()
			// Give other workers time to clean up
			<-time.After(3 * time.Second)
			relayProc, _ := os.FindProcess(os.Getpid())
			relayProc.Kill()
			return nil
		}
	}

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
			restarts := 0

			for {
				worker, config, err := factory()
				if err != nil {
					// It is unrecoverable if we cannot construct one of our workers
					return err
				}

				if !config.Enabled {
					return nil
				}

				log.WithFields(logrus.Fields{
					"restarts": restarts,
					"worker":   worker.Name(),
				}).Debug("Starting worker")
				err = wp.runWorker(ctx, worker)

				if err == WorkerDeadlocked {
					log.WithField(
						"worker",
						worker.Name(),
					).Error("The worker's goroutines are deadlocked. Please fix")
					return onDeadlock()
				} else {
					log.WithError(err).WithField("worker", worker.Name()).Debug("Worker terminated")
				}

				select {
				case <-ctx.Done():
					return ctx.Err()
				default:
					<-time.After(time.Duration(config.RestartDelay) * time.Second)
					restarts += 1
				}
			}
		})
	}

	return eg.Wait()
}

func (wp WorkerPool) defaultLogger() *logrus.Entry {
	return logrus.WithField("source", "WorkerPool")
}
