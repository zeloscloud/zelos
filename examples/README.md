# Zelos Examples

This directory contains examples demonstrating how to use the Zelos tracing system.

## Available Examples

### [hello-world](./hello-world.rs)

A minimal example that publishes a single data point to the Zelos system.

**What it demonstrates:**
- Basic setup of `TraceRouter` and `TracePublishClient`
- Creating event schemas with `TraceSource`
- Publishing a single event with typed fields
- Proper connection handling and cleanup

**Run it:**
```bash
cargo run --example hello-world
```

## Running Examples

All examples can be run from the workspace root:

```bash
# Run a specific example
cargo run --example <example-name>

# Build all examples
cargo build --examples

# Run with custom agent URL
ZELOS_URL=grpc://your-agent:2300 cargo run --example hello-world
```

## Prerequisites

Most examples require:
- A Zelos agent running (default: `grpc://127.0.0.1:2300`)
- Rust toolchain installed
- Network access to the agent

## Environment Variables

- `ZELOS_URL`: URL of the Zelos agent (default: `grpc://127.0.0.1:2300`)

## Contributing

When adding new examples:

1. Create a new `.rs` file under `examples/`
2. Add the example to the `[[example]]` section in `crates/zelos/Cargo.toml`
3. Add any necessary dependencies to the zelos crate
4. Update this README to document the new example
