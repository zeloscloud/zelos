#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import random
import time

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("sensors")

    temperature_event = source.add_event(
        "temperature",
        [zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64, "°C")],
    )

    while True:
        temperature = 20.0 + random.uniform(-2.0, 2.0)
        temperature_event.log(value=temperature)
        time.sleep(1.0)  # 1 Hz


if __name__ == "__main__":
    main()
