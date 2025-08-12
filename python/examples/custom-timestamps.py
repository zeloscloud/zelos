#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import time

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("replay")

    measurement = source.add_event(
        "measurement",
        [zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64)],
    )

    # Method 1: Current timestamp (automatic)
    measurement.log(value=1.0)

    # Method 2: Specific timestamp using log_at
    specific_time_ns = 1699564234567890123
    measurement.log_at(specific_time_ns, value=2.0)

    # Method 3: Calculated timestamp
    past_time_ns = time.time_ns() - 60 * 1_000_000_000
    measurement.log_at(past_time_ns, value=3.0)

    # Method 4: Synchronized timestamps
    for _ in range(3):
        timestamp_ns = time.time_ns() + 100_000  # offset 100 µs
        measurement.log_at(timestamp_ns, value=4.0)
        time.sleep(0.1)


if __name__ == "__main__":
    main()
