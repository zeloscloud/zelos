#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import math
import time

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("high_freq")

    data_event = source.add_event(
        "data",
        [zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64, "V")],
    )

    period_s = 0.001  # 1 kHz
    start_time = time.time()
    next_time = start_time

    while True:
        elapsed = time.time() - start_time
        value = math.sin(2.0 * math.pi * 100.0 * elapsed)
        data_event.log(value=value)

        next_time += period_s
        sleep_duration = next_time - time.time()
        if sleep_duration > 0:
            time.sleep(sleep_duration)


if __name__ == "__main__":
    main()
