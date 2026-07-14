// Register and serve custom actions to a Zelos agent.
//
// This example builds an ActionsRegistry with two actions and serves them to a
// Zelos agent over gRPC:
//   - add — adds two numbers and returns their sum (Done), schema built with the
//     ergonomic schema builder
//   - check_threshold — compares a value against a threshold (Pass/Fail)
//
// Once running, the actions appear on the agent as go-example/add and
// go-example/check_threshold.
//
// Run with `just example go actions` (optionally pass a custom agent URL).
package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/zeloscloud/zelos/go"
)

func main() {
	ctx := context.Background()

	// Configuration
	url := os.Getenv("ZELOS_URL")
	if url == "" {
		url = "grpc://127.0.0.1:2300"
	}
	log.Printf("Connecting to Zelos agent at: %s", url)

	// Register the actions under simple paths.
	registry := zelos.NewActionsRegistry()
	registry.Register("add", newAddAction())
	registry.Register("check_threshold", newCheckThresholdAction())

	// Set up the actions client.
	config := zelos.DefaultActionsClientConfig()
	config.URL = url
	client := zelos.NewActionsClient(ctx, "go-example", registry, config)

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

	log.Println("Serving actions as 'go-example/add' and 'go-example/check_threshold'")
	log.Println("Press Ctrl-C to stop.")

	// Block until interrupted.
	sig := make(chan os.Signal, 1)
	signal.Notify(sig, os.Interrupt, syscall.SIGTERM)
	<-sig

	log.Println("Shutting down...")
	if err := client.Close(); err != nil {
		log.Printf("Error closing client: %v", err)
	}
}

// newAddAction adds two numbers and returns their sum with a Done status.
func newAddAction() zelos.Action {
	schema := zelos.NewActionSchema("Add Numbers", "Add two numbers and return their sum").
		Number("x", zelos.Title("X"), zelos.Description("First number"), zelos.Required()).
		Number("y", zelos.Title("Y"), zelos.Description("Second number"), zelos.Required())

	return zelos.NewActionFromSchema(schema, func(ctx context.Context, params json.RawMessage) (*zelos.ActionResult, error) {
		var p struct {
			X *float64 `json:"x"`
			Y *float64 `json:"y"`
		}
		if err := json.Unmarshal(params, &p); err != nil {
			return nil, fmt.Errorf("invalid params: %w", err)
		}
		if p.X == nil || p.Y == nil {
			return nil, fmt.Errorf("missing or invalid 'x' or 'y'")
		}
		return zelos.ActionResultDone(map[string]any{"sum": *p.X + *p.Y}), nil
	})
}

// newCheckThresholdAction returns Pass when value <= threshold, otherwise Fail.
func newCheckThresholdAction() zelos.Action {
	schema := zelos.NewActionSchema("Check Threshold", "Check whether a value is within a threshold").
		Number("value", zelos.Title("Value"), zelos.Required()).
		Number("threshold", zelos.Title("Threshold"), zelos.Required())

	return zelos.NewActionFromSchema(schema, func(ctx context.Context, params json.RawMessage) (*zelos.ActionResult, error) {
		var p struct {
			Value     *float64 `json:"value"`
			Threshold *float64 `json:"threshold"`
		}
		if err := json.Unmarshal(params, &p); err != nil {
			return nil, fmt.Errorf("invalid params: %w", err)
		}
		if p.Value == nil || p.Threshold == nil {
			return nil, fmt.Errorf("missing or invalid 'value' or 'threshold'")
		}
		result := map[string]any{"value": *p.Value, "threshold": *p.Threshold}
		if *p.Value <= *p.Threshold {
			return zelos.ActionResultPass(result), nil
		}
		return zelos.ActionResultFail(result), nil
	})
}
