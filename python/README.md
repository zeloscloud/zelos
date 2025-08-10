# Zelos Python examples

Each example connects to a running Zelos agent via `ZELOS_URL` (default `grpc://127.0.0.1:2300`).

## How to run

From the repo root, using the top-level Justfile:

```bash
# List Python examples
just examples python

# Run one example (optional URL overrides default)
just example python hello-world
just example python hello-world grpc://127.0.0.1:2300
```

Notes:
- Long-running examples (streams) can be interrupted with Ctrl+C.
- The Nix dev shell provides Python and uv; outside the shell, install uv if needed.
