# Zelos Go examples

Standalone examples for publishing traces and serving actions with the Go SDK.

## Build and run

Ensure a Zelos agent/app is running and reachable.

From the `go` directory:

```bash
# Build every Go example
go build ./examples/...

# Build or run one example
go build ./examples/actions
ZELOS_URL=grpc://127.0.0.1:2300 go run ./examples/<example-name>
```

Or use the Justfile from the repository root:

```bash
# List Go examples
just examples go

# Run one example
just example go hello-world
just example go hello-world grpc://127.0.0.1:2300
```

## Actions

```bash
# Basic schema, registration, and Pass/Fail results
just example go actions

# Cooperative cancellation
just example go actions-advanced
```

While `actions-advanced` is running, invoke its long task from another terminal:

```bash
zelos actions execute go-advanced/long_task --params '{"seconds":30}'
```

Press Ctrl-C in the provider to cancel the in-flight action.
