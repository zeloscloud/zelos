# Zelos

A distributed tracing system built in Rust.

## Crates

- `zelos` - Main crate that re-exports all functionality
- `zelos-proto` - Protocol buffer definitions and gRPC types
- `zelos-trace` - Core tracing functionality
- `zelos-trace-grpc` - gRPC client and server implementations
- `zelos-trace-types` - Shared types and data structures

## Development

To work on a specific crate:

```bash
# Build all crates
cargo build

# Build a specific crate
cargo build -p zelos-trace

# Run tests for a specific crate
cargo test -p zelos-trace

# Run the publisher example
cargo run -p zelos-trace-grpc --bin zelos-trace-pub
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
