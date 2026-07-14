# Releasing the SDKs

Run release commands from a clean checkout of `main` after the release-preparation
pull request is merged.

## Verify

```bash
cargo build --workspace --examples
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

cd go
go build ./...
go vet ./...
go test -race ./...
```

## Publish Rust crates

Publish in dependency order. Wait for each crate version to appear on crates.io
before verifying or publishing the next dependent crate.

```bash
# 1. Shared protocol API
cargo publish -p zelos-proto --dry-run
cargo publish -p zelos-proto

# 2. Crates that depend on zelos-proto 0.0.2
cargo publish -p zelos-trace-grpc --dry-run
cargo publish -p zelos-trace-grpc

cargo publish -p zelos-actions --dry-run
cargo publish -p zelos-actions

# 3. Top-level SDK
cargo publish -p zelos --dry-run
cargo publish -p zelos
```

Cargo verifies packaged crates using dependencies downloaded from crates.io, not
workspace path dependencies. A dependent crate's dry run will therefore fail
until the preceding version is published and visible in the crates.io index.

## Tag releases

Tag the merge commit only after all Rust crates publish successfully:

```bash
git tag -a v0.0.2 -m "Zelos SDK v0.0.2"
git tag -a go/v0.0.2 -m "Go SDK v0.0.2"
git push origin v0.0.2 go/v0.0.2
```

The Go module is released by the `go/v0.0.2` tag; there is no separate upload.
