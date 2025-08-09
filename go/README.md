# Zelos Go SDK

Publish trace events from Go applications to a running Zelos agent/app.

## Installation

```bash
# For latest version
go get github.com/zeloscloud/zelos/go@latest

# For specific version
go get github.com/zeloscloud/zelos/go@v0.0.1
```

**Note**: Due to Go module limitations with subdirectory modules, version tags work differently. For production use, we recommend:
- Using `@latest` for the most recent version
- Using specific commit hashes for reproducible builds
- Or pinning the version in your `go.mod` file

## Quick Start

```go
package main

import (
    "context"
    "log"
    "time"
    "github.com/zeloscloud/zelos/go"
)

func main() {
    ctx := context.Background()

    // Set up router and client
    router, sender, receiver := zelos.NewTraceRouter(ctx)
    config := zelos.DefaultTracePublishClientConfig()
    client := zelos.NewTracePublishClient(ctx, receiver, config)

    // Start client
    go func() {
        if err := client.Run(); err != nil && err != context.Canceled {
            log.Printf("Client error: %v", err)
        }
    }()

    // Wait for connection
    if err := client.WaitUntilConnected(5 * time.Second); err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }

    // Create trace source
    source, err := zelos.NewTraceSource("my-app", sender)
    if err != nil {
        log.Fatal(err)
    }
    defer source.Close()

    // Register event schema
    event, err := source.BuildEvent("user_action").
        AddStringField("user_id", nil).
        AddUint64Field("timestamp", stringPtr("ns")).
        Build()
    if err != nil {
        log.Fatal(err)
    }

    // Emit event
    err = event.Build().
        TryInsertString("user_id", "user123").
        TryInsertUint64("timestamp", uint64(zelos.NowTimeNs())).
        Emit()
    if err != nil {
        log.Fatal(err)
    }

    log.Println("Event sent successfully!")
}

func stringPtr(s string) *string { return &s }
```

## Basic Usage

### 1. Create TraceSource

```go
source, err := zelos.NewTraceSource("service-name", sender)
defer source.Close() // Important: emits segment end
```

### 2. Register Event Schema

```go
event, err := source.BuildEvent("event_name").
    AddStringField("field1", nil).
    AddUint64Field("field2", stringPtr("ms")).
    AddBooleanField("field3", nil).
    Build()
```

### 3. Emit Events

```go
err = event.Build().
    TryInsertString("field1", "value").
    TryInsertUint64("field2", 1000).
    TryInsertBoolean("field3", true).
    Emit()
```

## Supported Field Types

- `AddStringField(name, unit)`
- `AddInt8Field(name, unit)` through `AddInt64Field(name, unit)`
- `AddUint8Field(name, unit)` through `AddUint64Field(name, unit)`
- `AddFloat32Field(name, unit)` and `AddFloat64Field(name, unit)`
- `AddBooleanField(name, unit)`
- `AddBinaryField(name, unit)`
- `AddTimestampNsField(name, unit)`

## Configuration

### Environment Variables

Set `ZELOS_URL` to specify the agent URL (default: `grpc://127.0.0.1:2300`)

### Custom Configuration

```go
config := &zelos.TracePublishClientConfig{
    URL:            "grpc://my-agent:2300",
    BatchSize:      1000,
    BatchTimeout:   100 * time.Millisecond,
    ReconnectDelay: 1000 * time.Millisecond,
}
client := zelos.NewTracePublishClient(ctx, receiver, config)
```

## Error Handling

```go
// Connection errors
if err := client.WaitUntilConnected(timeout); err != nil {
    log.Printf("Connection failed: %v", err)
}

// Schema validation errors
if err := builder.TryInsertString("field", "value"); err != nil {
    log.Printf("Validation error: %v", err)
}
```

## Building from Source

```bash
# Generate protobuf files
./generate.sh

# Build
go build -v ./...
```

## Examples in this repo

See `go/examples` and the top-level Justfile for publisher-only examples mirroring the Rust ones.

Run directly with Go:

```bash
ZELOS_URL=grpc://127.0.0.1:2300 go run go/examples/hello-world
```

Or use the unified helpers from the repo root:

```bash
just examples go
just example go hello-world grpc://127.0.0.1:2300
just examples-once go grpc://127.0.0.1:2300
```
