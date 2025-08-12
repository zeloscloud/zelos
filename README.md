# Zelos

A distributed tracing system built in Rust.

## Repository layout
- `crates/`
  - `zelos` — Meta crate re-exporting top-level APIs
  - `zelos-proto` — Protobuf definitions and generated types
  - `zelos-trace` — Core trace model and logic
  - `zelos-trace-grpc` — gRPC publish/subscribe client
  - `zelos-trace-types` — Shared types
- `examples/` — Rust examples
- `go/` — Go client, examples, generated stubs
- `python/` — Python examples (zelos-sdk pypi package)

## Quick start
Recommended: use the Nix dev shell. You can also run without Nix if you already have the toolchains.

### Nix + direnv
1) Install Nix and direnv
```bash
# Install Nix
sh <(curl -L https://nixos.org/nix/install) --daemon

# Install direnv
brew install direnv        # macOS
sudo apt-get install direnv -y  # Debian/Ubuntu

# Add to your shell rc (bash example)
echo 'eval "$(direnv hook bash)"' >> ~/.bashrc
source ~/.bashrc
```

2) Enter the dev shell (auto-activated with direnv)
```bash
cd zelos
# First time only
direnv allow
```
This provides Rust, Go, protoc (+ plugins), uv, ruff, treefmt and more.

3) Build and test
```bash
just build
just test
```

## Common commands
Use the top-level `Justfile`.

```bash
# List all recipes
just

# Build, check, lint, test
just build
just check
just clippy
just test

# Formatting
just fmt         # format all supported languages (treefmt)
just fmt-check   # check-only
just fix         # format and fix all
```

### Examples
Ensure a Zelos agent/app is reachable at your URL (default `grpc://127.0.0.1:2300`).

List examples for a language:
```bash
just examples rust
just examples go
just examples python
```

Run one example (optional URL overrides default):
```bash
just example rust hello-world
just example go hello-world
just example python hello-world
# with custom agent URL
just example rust hello-world grpc://127.0.0.1:2300
```

## Protobuf code generation

Regenerate Go stubs (when proto files change):
```bash
just proto-go
```
Outputs go to `go/` from sources in `crates/zelos-proto/proto/`.

## Developing
- Rust workspace: standard Cargo workflow (`just build`, `just test`, `just clippy`).
- Formatting/linting: `just fmt`, `just fmt-check`, `just fix`.
- Python examples run with `uv`
- Go examples run with the system `go`

## License
Licensed under either of:
- Apache License, Version 2.0 — see `LICENSE-APACHE` or https://www.apache.org/licenses/LICENSE-2.0
- MIT license — see `LICENSE-MIT` or https://opensource.org/licenses/MIT

At your option.

