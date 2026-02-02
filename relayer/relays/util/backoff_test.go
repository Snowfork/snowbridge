package util

import (
	"context"
	"testing"
	"time"
)

func TestScheduleConfig_Validate(t *testing.T) {
	tests := []struct {
		name    string
		config  ScheduleConfig
		wantErr bool
	}{
		{
			name: "valid config",
			config: ScheduleConfig{
				MinBackoffSeconds:   5,
				MaxBackoffSeconds:   30,
				JitterMs:            3000,
				MaxParallelMessages: 5,
			},
			wantErr: false,
		},
		{
			name: "min equals max is valid",
			config: ScheduleConfig{
				MinBackoffSeconds:   10,
				MaxBackoffSeconds:   10,
				JitterMs:            0,
				MaxParallelMessages: 1,
			},
			wantErr: false,
		},
		{
			name: "missing backoff config",
			config: ScheduleConfig{
				MinBackoffSeconds:   0,
				MaxBackoffSeconds:   0,
				MaxParallelMessages: 5,
			},
			wantErr: true,
		},
		{
			name: "max less than min",
			config: ScheduleConfig{
				MinBackoffSeconds:   30,
				MaxBackoffSeconds:   5,
				MaxParallelMessages: 5,
			},
			wantErr: true,
		},
		{
			name: "zero parallel messages",
			config: ScheduleConfig{
				MinBackoffSeconds:   5,
				MaxBackoffSeconds:   30,
				MaxParallelMessages: 0,
			},
			wantErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.config.Validate()
			if (err != nil) != tt.wantErr {
				t.Errorf("Validate() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}

func TestScheduleConfig_RandomBackoff(t *testing.T) {
	config := ScheduleConfig{
		MinBackoffSeconds:   5,
		MaxBackoffSeconds:   10,
		JitterMs:            1000,
		MaxParallelMessages: 1,
	}

	// Run multiple times to check randomness and bounds
	for i := 0; i < 100; i++ {
		backoff := config.RandomBackoff()

		minExpected := time.Duration(config.MinBackoffSeconds) * time.Second
		maxExpected := time.Duration(config.MaxBackoffSeconds)*time.Second + time.Duration(config.JitterMs)*time.Millisecond

		if backoff < minExpected {
			t.Errorf("RandomBackoff() = %v, want >= %v", backoff, minExpected)
		}
		if backoff > maxExpected {
			t.Errorf("RandomBackoff() = %v, want <= %v", backoff, maxExpected)
		}
	}
}

func TestScheduleConfig_RandomBackoff_MinEqualsMax(t *testing.T) {
	config := ScheduleConfig{
		MinBackoffSeconds:   10,
		MaxBackoffSeconds:   10,
		JitterMs:            0,
		MaxParallelMessages: 1,
	}

	backoff := config.RandomBackoff()
	expected := 10 * time.Second

	if backoff != expected {
		t.Errorf("RandomBackoff() = %v, want %v", backoff, expected)
	}
}

func TestScheduleConfig_WaitWithBackoff_ContextCancellation(t *testing.T) {
	config := ScheduleConfig{
		MinBackoffSeconds:   60, // Long backoff
		MaxBackoffSeconds:   120,
		JitterMs:            0,
		MaxParallelMessages: 1,
	}

	ctx, cancel := context.WithCancel(context.Background())

	// Cancel immediately
	cancel()

	start := time.Now()
	err := config.WaitWithBackoff(ctx)
	elapsed := time.Since(start)

	if err != context.Canceled {
		t.Errorf("WaitWithBackoff() error = %v, want %v", err, context.Canceled)
	}

	// Should return immediately on cancelled context
	if elapsed > 100*time.Millisecond {
		t.Errorf("WaitWithBackoff() took %v, expected immediate return on cancelled context", elapsed)
	}
}

func TestScheduleConfig_WaitWithBackoff_CompletesNormally(t *testing.T) {
	config := ScheduleConfig{
		MinBackoffSeconds:   0, // No backoff for quick test
		MaxBackoffSeconds:   0,
		JitterMs:            100, // Just 100ms jitter
		MaxParallelMessages: 1,
	}

	ctx := context.Background()

	start := time.Now()
	err := config.WaitWithBackoff(ctx)
	elapsed := time.Since(start)

	if err != nil {
		t.Errorf("WaitWithBackoff() error = %v, want nil", err)
	}

	// Should complete within the jitter time plus some margin
	if elapsed > 200*time.Millisecond {
		t.Errorf("WaitWithBackoff() took %v, expected < 200ms", elapsed)
	}
}
