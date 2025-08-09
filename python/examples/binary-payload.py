#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import os

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("binary-demo")

    event = source.add_event(
        "binary_payload",
        [
            zelos.TraceEventFieldMetadata("description", zelos.DataType.String),
            zelos.TraceEventFieldMetadata("payload", zelos.DataType.Binary),
        ],
    )

    # Example: send a small file or synthetic payload
    payload = os.urandom(64)
    event.log(description="random-bytes", payload=payload)


if __name__ == "__main__":
    main()
