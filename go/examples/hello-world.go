package main

import (
	"context"
	"log"
	"os"
	"time"

	"github.com/zeloscloud/zelos"
)

func main() {
	ctx := context.Background()

	// Configuration
	url := os.Getenv("ZELOS_URL")
	if url == "" {
		url = "grpc://127.0.0.1:2300"
	}
	log.Printf("Connecting to Zelos agent at: %s", url)

	// Set up the router (similar to Rust's TraceRouter::new)
	router, sender, receiver := zelos.NewTraceRouter(ctx)
	_ = router // We'll use this later when we implement subscription

	// Set up the publish client
	config := zelos.DefaultTracePublishClientConfig()
	config.URL = url
	client := zelos.NewTracePublishClient(ctx, receiver, config)

	// Start the client's processing task (similar to tokio::spawn in Rust)
	go func() {
		if err := client.Run(); err != nil && err != context.Canceled {
			log.Printf("Client error: %v", err)
		}
	}()

	// Wait for the client to connect
	if err := client.WaitUntilConnected(5 * time.Second); err != nil {
		log.Fatalf("Failed to connect to agent: %v", err)
	}
	log.Printf("Connected to agent at %s", url)

	// Create a TraceSource (similar to Rust's TraceSource::new)
	source, err := zelos.NewTraceSource("hello-world-go", sender)
	if err != nil {
		log.Fatalf("Failed to create trace source: %v", err)
	}
	defer func() {
		if err := source.Close(); err != nil {
			log.Printf("Error closing source: %v", err)
		}
	}()

	// Build and register a 'hello' event (similar to Rust's build_event pattern)
	helloEvent, err := source.BuildEvent("hello").
		AddUint64Field("count", nil).
		AddUint64Field("timestamp", stringPtr("ns")).
		Build()
	if err != nil {
		log.Fatalf("Failed to register hello event: %v", err)
	}

	// Publish a single hello message (similar to Rust's try_insert pattern)
	log.Println("Publishing hello message...")
	builder, err := helloEvent.Build().TryInsertUint64("count", 1)
	if err != nil {
		log.Fatalf("Failed to insert count: %v", err)
	}

	builder, err = builder.TryInsertUint64("timestamp", uint64(zelos.NowTimeNs()))
	if err != nil {
		log.Fatalf("Failed to insert timestamp: %v", err)
	}

	if err := builder.Emit(); err != nil {
		log.Fatalf("Failed to emit hello event: %v", err)
	}

	log.Println("Successfully published hello message!")
	log.Println("Check your Zelos App to see the data.")

	// Give a moment for the message to be sent
	time.Sleep(1 * time.Second)

	// Clean up
	if err := client.Close(); err != nil {
		log.Printf("Error closing client: %v", err)
	}
}

// Helper function to create string pointer
func stringPtr(s string) *string {
	return &s
}
