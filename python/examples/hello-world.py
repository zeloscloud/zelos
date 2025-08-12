#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import os
import time

import zelos_sdk as zelos


def main() -> None:
    url = os.getenv("ZELOS_URL", "grpc://127.0.0.1:2300")
    print(f"Connecting to Zelos agent at: {url}")

    # Initialize SDK (expects ZELOS_URL)
    zelos.init()

    source = zelos.TraceSource("hello-world-example")

    hello_event = source.add_event(
        "hello",
        [
            zelos.TraceEventFieldMetadata("count", zelos.DataType.UInt64),
            zelos.TraceEventFieldMetadata("timestamp", zelos.DataType.Int64, "ns"),
        ],
    )

    print("Publishing hello message...")
    hello_event.log(count=1, timestamp=int(time.time_ns()))

    # Give a brief moment for delivery
    time.sleep(0.25)
    print("Successfully published hello message! Check your Zelos App.")


if __name__ == "__main__":
    main()
