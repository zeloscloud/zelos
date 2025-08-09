# Zelos Rust examples

Examples showing how to use the Zelos Rust client APIs to publish trace events. Each example is standalone.

## How to run

All examples are wired to the `zelos` crate. Ensure a Zelos agent/app is running and reachable. Run any example with:

```bash
cargo run -p zelos --example <example-name>
```

Set the agent URL via `ZELOS_URL` (defaults to `grpc://127.0.0.1:2300`). You can also use the Justfile helpers:

```bash
# List examples
just examples rust
just examples python

# Run a specific example
just example rust hello-world grpc://127.0.0.1:2300
just example python hello-world grpc://127.0.0.1:2300

# Run all finite Rust examples once
just examples-once rust grpc://127.0.0.1:2300
just examples-once python grpc://127.0.0.1:2300
```

## Examples

- **hello-world**: Minimal publisher that connects to an agent and emits one `hello` event.
  - Run: `just run-example hello-world`

- **grpc-publish-config**: Demonstrates client configuration (batch size, timeout, reconnect) and connection status.
  - Run: `just run-example grpc-publish-config`

- **all-types**: Defines an event that exercises every `DataType` and emits one event with example values.
  - Run: `just run-example all-types`

- **async-emit**: Uses async variants of event creation and emission.
  - Run: `just run-example async-emit`

- **event-schema-and-values**: Demonstrates schema definition plus value tables for friendly categorical decoding.
  - Run: `just run-example event-schema-and-values`

- **basic-stream**: 1 Hz temperature stream with a simple schema. Long running.
  - Run: `just run-basic-stream`

- **high-frequency**: 1 kHz sine wave stream with precise timing. Long running.
  - Run: `just run-high-frequency`

- **burst-stream**: Start/sample/end burst logging pattern every 5s. Long running.
  - Run: `just run-burst-stream`

- **multi-sources**: Multiple sources streaming in parallel at different rates.
  - Run: `just run-multi-sources`

- **custom-timestamps**: Emit using automatic, specific, and computed timestamps.
  - Run: `just run-example custom-timestamps`

- **state-machine**: State transitions with value tables for readable state names.
  - Run: `just run-example state-machine`

- **sensor-array**: Emit an array of sensor values in a single event.
  - Run: `just run-example sensor-array`

- **publish-status**: Observe connection/publish status updates while emitting.
  - Run: `just run-example publish-status`

- **binary-payload**: Emit binary data alongside descriptive fields.
  - Run: `just run-example binary-payload`

- **async-schema**: Register schema and emit asynchronously.
  - Run: `just run-example async-schema`

## Notes

- The gRPC URL defaults to `grpc://127.0.0.1:2300`. Override with `ZELOS_URL`.
