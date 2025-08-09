#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import time
from typing import Iterable

import zelos_sdk as zelos


def handle_burst_event(source: "zelos.TraceSource", trigger_name: str, samples: Iterable[float]) -> None:
    start_time = time.time()

    source.log(
        "event_start",
        {"trigger": trigger_name, "timestamp_ms": int(start_time * 1000)},
    )

    sample_count = 0
    for index, value in enumerate(samples):
        source.log("event_sample", {"value": float(value), "index": int(index)})
        sample_count = index + 1

    duration_ms = (time.time() - start_time) * 1000.0
    source.log(
        "event_end",
        {"duration_ms": float(duration_ms), "sample_count": int(sample_count)},
    )


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("events")

    source.add_event(
        "event_start",
        [
            zelos.TraceEventFieldMetadata("trigger", zelos.DataType.String),
            zelos.TraceEventFieldMetadata("timestamp_ms", zelos.DataType.Int64),
        ],
    )

    source.add_event(
        "event_sample",
        [
            zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64),
            zelos.TraceEventFieldMetadata("index", zelos.DataType.UInt32),
        ],
    )

    source.add_event(
        "event_end",
        [
            zelos.TraceEventFieldMetadata("duration_ms", zelos.DataType.Float64),
            zelos.TraceEventFieldMetadata("sample_count", zelos.DataType.UInt32),
        ],
    )

    while True:
        time.sleep(5.0)
        burst_data = [i * 0.1 for i in range(100)]
        handle_burst_event(source, "threshold_exceeded", burst_data)


if __name__ == "__main__":
    main()
