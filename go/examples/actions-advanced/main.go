// Demonstrate application-driven cancellation of an in-flight action.
//
// This example serves a single long-running action, long_task, that sleeps for
// a number of seconds while watching its context. Because the handler's context
// derives from the context passed to NewActionsClient, cancelling that context
// (on Ctrl-C) reaches the running handler, which stops cooperatively and logs
// "Long task cancelled".
//
// Try it:
//  1. Run the provider: `just example go actions-advanced`
//  2. In another terminal: `zelos actions execute go-advanced/long_task --params '{"seconds":30}'`
//  3. Back in the provider, press Ctrl-C while the invocation is active.
//
// The action appears on the agent as go-advanced/long_task.
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"os/signal"
	"sync"
	"syscall"
	"time"

	"github.com/zeloscloud/zelos/go"
)

func main() {
	// A cancellable context whose cancellation propagates to action handlers.
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Configuration
	url := os.Getenv("ZELOS_URL")
	if url == "" {
		url = "grpc://127.0.0.1:2300"
	}
	log.Printf("Connecting to Zelos agent at: %s", url)

	// Register the long-running action.
	var active sync.WaitGroup
	registry := zelos.NewActionsRegistry()
	registry.Register("long_task", newLongTaskAction(&active))

	// Set up the actions client with our cancellable context.
	config := zelos.DefaultActionsClientConfig()
	config.URL = url
	client := zelos.NewActionsClient(ctx, "go-advanced", registry, config)

	// Start the client's serving loop (reconnects automatically).
	go func() {
		if err := client.Run(); err != nil && err != context.Canceled {
			log.Printf("Client error: %v", err)
		}
	}()

	// Wait for the client to connect.
	if err := client.WaitUntilConnected(5 * time.Second); err != nil {
		log.Printf("Not connected yet (%v); will keep retrying in the background", err)
	} else {
		log.Printf("Connected to agent at %s", url)
	}

	log.Println("Serving action as 'go-advanced/long_task'")
	log.Println("Press Ctrl-C to stop (cancels any in-flight invocation).")

	// Block until interrupted.
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt, syscall.SIGTERM)
	<-sig

	// Cancel the context first so in-flight handlers observe ctx.Done(), then
	// close the client.
	log.Println("Shutting down...")
	cancel()
	if err := client.Close(); err != nil {
		log.Printf("Error closing client: %v", err)
	}
	active.Wait()
}

// newLongTaskAction sleeps for the requested number of seconds, returning early
// if its context is cancelled.
func newLongTaskAction(active *sync.WaitGroup) zelos.Action {
	schema := zelos.NewActionSchema("Long Task", "Sleep for the given number of seconds").
		Integer("seconds", zelos.Title("Seconds"), zelos.Minimum(1), zelos.Required())

	return zelos.NewActionFromSchema(schema, func(ctx context.Context, params json.RawMessage) (*zelos.ActionResult, error) {
		active.Add(1)
		defer active.Done()

		var p struct {
			Seconds *int64 `json:"seconds"`
		}
		if err := json.Unmarshal(params, &p); err != nil {
			return nil, fmt.Errorf("invalid params: %w", err)
		}
		if p.Seconds == nil || *p.Seconds <= 0 {
			return nil, fmt.Errorf("'seconds' must be a positive integer")
		}

		timer := time.NewTimer(time.Duration(*p.Seconds) * time.Second)
		defer timer.Stop()

		select {
		case <-ctx.Done():
			log.Println("Long task cancelled")
			return nil, ctx.Err()
		case <-timer.C:
			return zelos.ActionResultDone(map[string]any{"slept_seconds": *p.Seconds}), nil
		}
	})
}
