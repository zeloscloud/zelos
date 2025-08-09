#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import os

import zelos_sdk as zelos


def main() -> None:
    # Configure via environment (the SDK reads ZELOS_URL)
    url = os.getenv("ZELOS_URL", "grpc://127.0.0.1:2300")
    print(f"Using ZELOS_URL={url}")

    # Initialize SDK
    zelos.init()

    # Demonstrate simple publish after init
    source = zelos.TraceSource("config-demo")
    evt = source.add_event(
        "status",
        [zelos.TraceEventFieldMetadata("message", zelos.DataType.String)],
    )
    evt.log(message="client initialized")


if __name__ == "__main__":
    main()
