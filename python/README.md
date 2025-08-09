# Zelos Python examples

Each example connects to a running Zelos agent via `ZELOS_URL` (default `grpc://127.0.0.1:2300`).

## Setup

These scripts are uv-based and self-contained.

```bash
# Install uv (if needed)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## How to run

Option A:

```bash
python/examples/hello-world.py
```

Option B:

```bash
# From repo root
just examples python
just example python hello-world grpc://127.0.0.1:2300
just examples-once python grpc://127.0.0.1:2300
```

## Notes

- Long-running examples (streams) can be interrupted with Ctrl+C.
