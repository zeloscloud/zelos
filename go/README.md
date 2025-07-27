# Zelos Go SDK

A Go SDK for the Zelos distributed tracing system.

## Installation

```bash
go get github.com/zeloscloud/zelos/go
```

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

## Run example
```bash
go run examples/hello-world.go
```

## Troubleshooting

**Connection refused**: Check if Zelos agent is running at the configured URL

**Field validation errors**: Ensure field names and types match the registered schema

**Build errors**: Run `./generate.sh` to regenerate protobuf files
