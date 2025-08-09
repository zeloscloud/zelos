#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import random

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("sensor-array")

    fields = [
        zelos.TraceEventFieldMetadata(f"sensor_{i}", zelos.DataType.Float32)
        for i in range(16)
    ]
    sensor_array = source.add_event("array", fields)

    values = {f"sensor_{i}": float(random.uniform(-1.0, 1.0)) for i in range(16)}
    sensor_array.log(**values)


if __name__ == "__main__":
    main()
