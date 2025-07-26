# Zelos Go SDK

A Go SDK for the Zelos distributed tracing system, providing a simple and efficient way to instrument your Go applications with distributed tracing capabilities. This SDK closely mirrors the architecture and patterns of the Rust SDK.

## Features

- **Rust-Compatible Architecture**: Closely mirrors the Rust SDK's design patterns and API
- **UUID-based Segments**: Proper segment lifecycle management with UUIDs
- **Schema Registration**: Type-safe event schemas with validation
- **Builder Pattern API**: Fluent API for creating and emitting events
- **Connection Management**: Automatic reconnection with status tracking
- **Channel-based IPC**: Efficient message passing between components
- **Proper Error Handling**: Go-style error handling with detailed error messages
- **Thread Safe**: All operations are thread-safe for concurrent use

## Installation

```bash
go get github.com/zeloscloud/zelos
```

## Quick Start

```go
package main

import (
    "context"
    "log"
    "time"
    "github.com/zeloscloud/zelos"
)

func main() {
    ctx := context.Background()

    // Set up the router (similar to Rust's TraceRouter::new)
    router, sender, receiver := zelos.NewTraceRouter(ctx)

    // Set up the publish client
    config := zelos.DefaultTracePublishClientConfig()
    client := zelos.NewTracePublishClient(ctx, receiver, config)

    // Start the client's processing task
    go client.Run()

    // Wait for connection
    client.WaitUntilConnected(5 * time.Second)

    // Create a trace source (emits segment start automatically)
    source, err := zelos.NewTraceSource("my-app", sender)
    if err != nil {
        log.Fatal(err)
    }
    defer source.Close() // Emits segment end

    // Build and register an event schema
    event, err := source.BuildEvent("user_login").
        AddStringField("user_id", nil).
        AddUint64Field("timestamp", stringPtr("ns")).
        Build()
    if err != nil {
        log.Fatal(err)
    }

    // Emit an event with type validation
    builder, err := event.Build().TryInsertString("user_id", "user123")
    if err != nil {
        log.Fatal(err)
    }

    builder, err = builder.TryInsertUint64("timestamp", uint64(zelos.NowTimeNs()))
    if err != nil {
        log.Fatal(err)
    }

    err = builder.Emit()
    if err != nil {
        log.Fatal(err)
    }
}

func stringPtr(s string) *string { return &s }
```

## Core Concepts

### TraceSource

A `TraceSource` represents a trace segment with a unique UUID. It automatically emits segment start/end messages and manages event schemas.

```go
// Creates a new segment with UUID and emits segment start
source, err := zelos.NewTraceSource("my-service", sender)
if err != nil {
    log.Fatal(err)
}
defer source.Close() // Important: emits segment end
```

### Event Schema Registration

Events must be registered with a schema before use, providing type safety:

```go
event, err := source.BuildEvent("database_query").
    AddStringField("query", nil).
    AddUint64Field("duration_ns", stringPtr("ns")).
    AddBooleanField("success", nil).
    Build()
if err != nil {
    log.Fatal(err)
}
```

### Type-Safe Event Building

Events are built with type validation against the registered schema:

```go
builder, err := event.Build().TryInsertString("query", "SELECT * FROM users")
if err != nil {
    log.Fatal(err) // Type mismatch or unknown field
}

builder, err = builder.TryInsertUint64("duration_ns", 1500000)
if err != nil {
    log.Fatal(err)
}

err = builder.Emit()
if err != nil {
    log.Fatal(err)
}
```

### TraceRouter and Channels

The router uses Go channels for efficient IPC message passing:

```go
router, sender, receiver := zelos.NewTraceRouter(ctx)
// sender: for TraceSource to send messages
// receiver: for TracePublishClient to receive messages
```

### Connection Management

The publish client handles connection status and automatic reconnection:

```go
client := zelos.NewTracePublishClient(ctx, receiver, config)

// Start processing in background
go func() {
    if err := client.Run(); err != nil && err != context.Canceled {
        log.Printf("Client error: %v", err)
    }
}()

// Wait for connection
if err := client.WaitUntilConnected(5 * time.Second); err != nil {
    log.Fatal(err)
}

// Check status
status := client.GetConnectionStatus()
fmt.Printf("Status: %s\n", status)
```

## Architecture Alignment with Rust SDK

| Rust Component | Go Equivalent | Notes |
|----------------|---------------|-------|
| `TraceSource::new()` | `NewTraceSource()` | Auto-generates UUID, emits segment start |
| `build_event().add_*_field()` | `BuildEvent().Add*Field()` | Schema registration with units |
| `event.build().try_insert_*()` | `event.Build().TryInsert*()` | Type-safe value insertion |
| `TraceRouter::new()` | `NewTraceRouter()` | Channel-based message routing |
| `TracePublishClient::new()` | `NewTracePublishClient()` | Connection management with status |
| Segment lifecycle | `defer source.Close()` | Automatic segment end emission |

## Supported Data Types

All Rust SDK data types are supported with proper Go mappings:

- **Integers**: `int8`, `int16`, `int32`, `int64`
- **Unsigned Integers**: `uint8`, `uint16`, `uint32`, `uint64`
- **Floats**: `float32`, `float64`
- **Other**: `string`, `bool`, `[]byte`, `int64` (timestamp)

## IPC Message Types

The SDK implements all IPC message types from the Rust version:

- `TraceSegmentStart` - Segment lifecycle begin
- `TraceSegmentEnd` - Segment lifecycle end
- `TraceEventSchema` - Event schema registration
- `TraceEvent` - Actual trace event data
- `TraceEventFieldNamedValues` - Named value mappings

## Error Handling

The SDK uses Go's standard error handling with detailed error messages:

```go
// Schema validation
if err := builder.TryInsert("field", value); err != nil {
    // Errors include: field not found, type mismatch
    log.Printf("Validation error: %v", err)
}

// Connection errors
if err := client.WaitUntilConnected(timeout); err != nil {
    log.Printf("Connection failed: %v", err)
}
```

## Configuration

### TracePublishClientConfig

```go
config := &zelos.TracePublishClientConfig{
    URL:            "grpc://localhost:2300",  // Agent URL
    BatchSize:      1000,                     // Messages per batch
    BatchTimeout:   100 * time.Millisecond,  // Max batch wait time
    ReconnectDelay: 1000 * time.Millisecond, // Delay between reconnects
}
```

### Environment Variables

- `ZELOS_URL` - The URL of the Zelos agent (default: `grpc://127.0.0.1:2300`)

## Examples

See the `examples/` directory for complete working examples:

- `hello-world.go` - Basic usage matching Rust example
- Demonstrates proper lifecycle management
- Shows type-safe event building patterns

## Thread Safety

All public APIs are thread-safe and designed for concurrent use across goroutines.

## Contributing

Contributions are welcome! Please ensure any changes maintain compatibility with the Rust SDK patterns.

## License

This project is licensed under the same terms as the Rust SDK.
