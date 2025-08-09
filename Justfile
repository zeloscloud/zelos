# Top-level Justfile for Zelos

set shell := ["bash", "-cu"]

# Defaults and example sets
default_url := "grpc://127.0.0.1:2300"

rust_examples_once := "hello-world all-types async-emit event-schema-and-values binary-payload custom-timestamps sensor-array grpc-publish-config publish-status state-machine async-schema"
rust_examples_stream := "basic-stream high-frequency burst-stream multi-sources"

go_examples_once := "hello-world all-types async-emit event-schema-and-values binary-payload custom-timestamps sensor-array grpc-publish-config publish-status state-machine"
go_examples_stream := "basic-stream high-frequency burst-stream multi-sources"

# Python examples (parity with Go set)
python_examples_once := "hello-world all-types async-emit event-schema-and-values binary-payload custom-timestamps sensor-array grpc-publish-config publish-status state-machine"
python_examples_stream := "basic-stream high-frequency burst-stream multi-sources"

# Default: show available recipes
default:
    @just --list

# Workspace-wide tasks
build:
    cargo build --workspace

check:
    cargo check --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

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
    ( cd go && \
      set -e; \
      command -v protoc >/dev/null 2>&1 || (echo "protoc not found. Install it first (e.g., 'brew install protobuf' or 'apt-get install -y protobuf-compiler')." >&2; exit 1); \
      if command -v go >/dev/null 2>&1; then \
        BIN="$$(go env GOBIN)"; [ -n "$$BIN" ] || BIN="$$(go env GOPATH)/bin"; \
        export PATH="$$PATH:$$BIN"; \
        command -v protoc-gen-go >/dev/null 2>&1 || (go install google.golang.org/protobuf/cmd/protoc-gen-go@latest && export PATH="$$PATH:$$BIN"); \
        command -v protoc-gen-go-grpc >/dev/null 2>&1 || (go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest && export PATH="$$PATH:$$BIN"); \
      fi; \
      command -v protoc-gen-go >/dev/null 2>&1 || (echo "protoc-gen-go not found in PATH" >&2; exit 1); \
      command -v protoc-gen-go-grpc >/dev/null 2>&1 || (echo "protoc-gen-go-grpc not found in PATH" >&2; exit 1); \
      protoc --go_out=. --go_opt=paths=source_relative \
             --go_opt=Mzeloscloud/trace/trace.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --go_opt=Mzeloscloud/trace/publish.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --go_opt=Mzeloscloud/trace/subscribe.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --go-grpc_out=. --go-grpc_opt=paths=source_relative \
             --go-grpc_opt=Mzeloscloud/trace/trace.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --go-grpc_opt=Mzeloscloud/trace/publish.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --go-grpc_opt=Mzeloscloud/trace/subscribe.proto=github.com/zeloscloud/zelos/go/zeloscloud/trace \
             --proto_path=crates/zelos-proto/proto \
             crates/zelos-proto/proto/zeloscloud/trace/*.proto )

# Optional: install protoc via package manager (best-effort)
setup-proto-tools:
    @if command -v protoc >/dev/null 2>&1; then \
        echo "protoc already installed: $$(protoc --version)"; \
    elif command -v brew >/dev/null 2>&1; then \
        echo "Installing protobuf via Homebrew..."; brew install protobuf; \
    elif command -v apt-get >/dev/null 2>&1; then \
        echo "Installing protobuf via apt-get..."; sudo apt-get update && sudo apt-get install -y protobuf-compiler; \
    else \
        echo "Please install protoc manually (https://grpc.io/docs/protoc-installation/)."; exit 1; \
    fi

# Optional: install Go and required protoc plugins (best-effort)
setup-go-tools:
    @if ! command -v go >/dev/null 2>&1; then \
        if command -v brew >/dev/null 2>&1; then \
            echo "Installing Go via Homebrew..."; brew install go; \
        elif command -v apt-get >/dev/null 2>&1; then \
            echo "Installing Go via apt-get..."; sudo apt-get update && sudo apt-get install -y golang-go; \
        else \
            echo "Go is not installed. Please install from https://go.dev/dl/ and ensure 'go' is on PATH."; exit 1; \
        fi; \
    fi; \
    BIN="$$(go env GOBIN)"; [ -n "$$BIN" ] || BIN="$$(go env GOPATH)/bin"; \
    echo "Installing protoc plugins to $$BIN"; \
    go install google.golang.org/protobuf/cmd/protoc-gen-go@latest; \
    go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest; \
    echo "If needed, add to PATH: export PATH=\"\$$PATH:$$BIN\""

# Examples (unified)
examples-list:
    @echo "Rust (finite):   {{rust_examples_once}}" && \
    echo "Rust (stream):  {{rust_examples_stream}}" && \
    echo "Go (finite):     {{go_examples_once}}" && \
    echo "Go (stream):    {{go_examples_stream}}" && \
    echo "Python (finite): {{python_examples_once}}" && \
    echo "Python (stream): {{python_examples_stream}}"

examples lang="all":
	@if [ "{{lang}}" = "rust" ]; then \
		echo "Rust (finite):   {{rust_examples_once}}"; \
		echo "Rust (stream):  {{rust_examples_stream}}"; \
	elif [ "{{lang}}" = "go" ]; then \
		echo "Go (finite):     {{go_examples_once}}"; \
		echo "Go (stream):    {{go_examples_stream}}"; \
	elif [ "{{lang}}" = "python" ]; then \
		echo "Python (finite): {{python_examples_once}}"; \
		echo "Python (stream): {{python_examples_stream}}"; \
	else \
		just examples-list; \
	fi

example lang name url="":
    @URL="{{url}}"; [ -n "$URL" ] || URL="{{default_url}}"; \
    if [ "{{lang}}" = "rust" ]; then \
        ZELOS_URL="$URL" cargo run -p zelos --example {{name}}; \
    elif [ "{{lang}}" = "go" ]; then \
        ( cd go && ZELOS_URL="$URL" go run ./examples/{{name}} ); \
    elif [ "{{lang}}" = "python" ]; then \
        ZELOS_URL="$URL" uv run python/examples/{{name}}.py; \
    else \
        echo "Unsupported language: {{lang}}" && exit 1; \
    fi

examples-once lang url="":
    @URL="{{url}}"; [ -n "$URL" ] || URL="{{default_url}}"; \
    if [ "{{lang}}" = "rust" ]; then \
        for ex in {{rust_examples_once}}; do ZELOS_URL="$URL" cargo run -p zelos --example "$ex"; done; \
    elif [ "{{lang}}" = "go" ]; then \
        ( cd go && for ex in {{go_examples_once}}; do ZELOS_URL="$URL" go run ./examples/"$ex"; done ); \
    elif [ "{{lang}}" = "python" ]; then \
        for ex in {{python_examples_once}}; do ZELOS_URL="$URL" uv run python/examples/"$ex".py; done; \
    else \
        echo "Unsupported language: {{lang}}" && exit 1; \
    fi
