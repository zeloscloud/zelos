# Top-level Justfile for Zelos

# Defaults
default_url := "grpc://127.0.0.1:2300"

# Default: show available recipes
default:
    @just --list

# Workspace-wide tasks
build:
    cargo build --workspace

check:
    cargo check --workspace

fmt:
    treefmt .

fmt-check:
    treefmt --fail-on-change .

typos:
    typos .

fix:
    treefmt .
    typos -w .

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test *args:
    cargo test --workspace -- {{args}}

# Per-crate tests
# Usage: just test-crate zelos-trace
test-crate crate:
    cargo test -p {{crate}}

# Benches (if available)
bench:
    cargo bench -p zelos-trace-grpc

# Proto generation for Go (uses repo proto source)
proto-go:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v protoc >/dev/null 2>&1 || { echo "protoc not found" >&2; exit 1; }
    command -v protoc-gen-go >/dev/null 2>&1 || { echo "protoc-gen-go not found" >&2; exit 1; }
    command -v protoc-gen-go-grpc >/dev/null 2>&1 || { echo "protoc-gen-go-grpc not found" >&2; exit 1; }
    protoc \
      --go_out=go \
      --go_opt=paths=source_relative \
      --go_opt=Mzeloscloud/trace/trace.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --go_opt=Mzeloscloud/trace/publish.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --go_opt=Mzeloscloud/trace/subscribe.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --go-grpc_out=go \
      --go-grpc_opt=paths=source_relative \
      --go-grpc_opt=Mzeloscloud/trace/trace.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --go-grpc_opt=Mzeloscloud/trace/publish.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --go-grpc_opt=Mzeloscloud/trace/subscribe.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
      --proto_path=crates/zelos-proto/proto \
      crates/zelos-proto/proto/zeloscloud/trace/*.proto


# === Examples === #

examples lang:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{lang}}" = "rust" ]; then
      echo "Rust examples:";
      for f in examples/*.rs; do
        [ -f "$f" ] || continue
        base=$(basename "$f")
        name=${base%.rs}
        [ "$name" = "README" ] && continue
        echo "  - $name"
      done
    elif [ "{{lang}}" = "go" ]; then
      echo "Go examples:";
      for d in go/examples/*; do
        [ -d "$d" ] || continue
        echo "  - $(basename "$d")"
      done
    elif [ "{{lang}}" = "python" ]; then
      echo "Python examples:";
      for f in python/examples/*.py; do
        [ -f "$f" ] || continue
        echo "  - $(basename "${f%.py}")"
      done
    else
      echo "Unsupported language: {{lang}} (use: rust | go | python)" >&2
      exit 1
    fi

example lang name url="":
    @URL="{{url}}"; [ -n "$URL" ] || URL="{{default_url}}"; \
    if [ "{{lang}}" = "rust" ]; then \
      ZELOS_URL="$URL" cargo run -p zelos --example {{name}}; \
    elif [ "{{lang}}" = "go" ]; then \
      ( cd go && ZELOS_URL="$URL" go run ./examples/{{name}} ); \
    elif [ "{{lang}}" = "python" ]; then \
      ( cd python && ZELOS_URL="$URL" uv run examples/{{name}}.py ); \
    else \
      echo "Unsupported language: {{lang}}" && exit 1; \
    fi

