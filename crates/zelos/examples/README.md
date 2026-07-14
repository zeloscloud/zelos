# Zelos Rust examples

Examples showing how to use the Zelos Rust client APIs to publish trace events. Each example is standalone.

## Build and run

Ensure a Zelos agent/app is running and reachable.

Using Cargo directly from the repository root:
```bash
# Build every Rust example
cargo build -p zelos --features examples --examples

# Build or run one example
cargo build -p zelos --features examples --example actions
cargo run -p zelos --features examples --example <example-name>
```

Or use the Justfile:
```bash
# List Rust examples
just examples rust

# Run one
just example rust hello-world
just example rust hello-world grpc://127.0.0.1:2300
```

### Actions

```bash
# Basic schema, registration, and Pass/Fail results
just example rust actions

# Cooperative cancellation
just example rust actions-advanced
```

While `actions-advanced` is running, invoke its long task from another terminal:

```bash
zelos actions execute rust-advanced/long_task --params '{"seconds":30}'
```

Press Ctrl-C in the provider to cancel the in-flight action.

## Examples

- **actions**: Defines and serves basic actions with ordered input schemas.
  - Run: `just example rust actions`

- **actions-advanced**: Uses `ActionFn` and demonstrates cancellation of in-flight async work.
  - Run: `just example rust actions-advanced`

- **hello-world**: Minimal publisher that connects to an agent and emits one `hello` event.
  - Run: `just example rust hello-world`

- **grpc-publish-config**: Demonstrates client configuration (batch size, timeout, reconnect) and connection status.
  - Run: `just example rust grpc-publish-config`

- **all-types**: Defines an event that exercises every `DataType` and emits one event with example values.
  - Run: `just example rust all-types`

- **async-emit**: Uses async variants of event creation and emission.
  - Run: `just example rust async-emit`

- **event-schema-and-values**: Demonstrates schema definition plus value tables for friendly categorical decoding.
  - Run: `just example rust event-schema-and-values`

- **basic-stream**: 1 Hz temperature stream with a simple schema. Long running.
  - Run: `just example rust basic-stream`

- **high-frequency**: 1 kHz sine wave stream with precise timing. Long running.
  - Run: `just example rust high-frequency`

- **burst-stream**: Start/sample/end burst logging pattern every 5s. Long running.
  - Run: `just example rust burst-stream`

- **multi-sources**: Multiple sources streaming in parallel at different rates.
  - Run: `just example rust multi-sources`

- **custom-timestamps**: Emit using automatic, specific, and computed timestamps.
  - Run: `just example rust custom-timestamps`

- **state-machine**: State transitions with value tables for readable state names.
  - Run: `just example rust state-machine`

- **sensor-array**: Emit an array of sensor values in a single event.
  - Run: `just example rust sensor-array`

- **publish-status**: Observe connection/publish status updates while emitting.
  - Run: `just example rust publish-status`

- **binary-payload**: Emit binary data alongside descriptive fields.
  - Run: `just example rust binary-payload`

- **async-schema**: Register schema and emit asynchronously.
  - Run: `just example rust async-schema`

## Notes

- The gRPC URL defaults to `grpc://127.0.0.1:2300`. Override with `ZELOS_URL`.
