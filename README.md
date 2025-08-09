# Zelos

A distributed tracing system built in Rust.

## Crates

- `zelos` - Main crate that re-exports all functionality
- `zelos-proto` - Protocol buffer definitions and gRPC types
- `zelos-trace` - Core tracing functionality
- `zelos-trace-grpc` - gRPC client and server implementations
- `zelos-trace-types` - Shared types and data structures

## Development

### Prerequisites

- Rust toolchain (via rustup)
- just (command runner, optional but recommended)

Install on macOS:

```bash
# Install Rust (includes cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install just
# 1) Homebrew
brew install just
# 2) Cargo
cargo install just
```

Install on Linux:

```bash
# Install Rust (includes cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install just (choose one)
# 1) Your package manager (Debian/Ubuntu example)
sudo apt-get update && sudo apt-get install -y just || true
# 2) Cargo
cargo install just
```

Install on Windows (PowerShell):

```powershell
# Install Rust (includes cargo)
iwr https://sh.rustup.rs -UseBasicParsing | Invoke-Expression

# Install just via cargo
cargo install just
```

Notes:
- After installing Rust, ensure your shell environment is reloaded so `cargo` is on PATH (`source $HOME/.cargo/env`).
- We vendor protoc via prost; you do NOT need system `protoc`.

### Use the top-level Justfile for common tasks

```bash
# See available commands
just

# Build, check, lint, test
just build
just check
just fmt
just fmt-check
just clippy
just test

# List examples by language
just examples rust
just examples go
just examples python

# Run one example (language-agnostic)
just example rust hello-world grpc://127.0.0.1:2300
just example go hello-world grpc://127.0.0.1:2300
just example python hello-world grpc://127.0.0.1:2300

# Run all finite examples for a language
just examples-once rust grpc://127.0.0.1:2300
just examples-once go grpc://127.0.0.1:2300
just examples-once python grpc://127.0.0.1:2300
```

### Quickstart

```bash
# 1) Clone the repo
git clone https://github.com/zeloscloud/zelos.git
cd zelos

# 2) Build the workspace
just build

# 3) Ensure a Zelos agent/app is running and reachable at your URL (default below).

# 4) Run your first publisher examples
just example rust hello-world grpc://127.0.0.1:2300
just example go hello-world grpc://127.0.0.1:2300

# Optional: run all finite publishing examples
just examples-once rust grpc://127.0.0.1:2300
just examples-once go grpc://127.0.0.1:2300

### Run without just

Rust:
```bash
ZELOS_URL=grpc://127.0.0.1:2300 cargo run -p zelos --example hello-world
```

Go:
```bash
ZELOS_URL=grpc://127.0.0.1:2300 go run go/examples/hello-world
```

### Protobuf (Go) code generation

Only needed if you modify proto files and need to regenerate the Go stubs.

Requirements:
- protoc installed
  - macOS: `brew install protobuf`
  - Debian/Ubuntu: `sudo apt-get install -y protobuf-compiler`
- Go toolchain (recommended so the Just recipe can install plugins):
  - macOS: `brew install go`

Generate Go protobufs:

```bash
just proto-go
```

If the recipe reports missing plugins:

```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
