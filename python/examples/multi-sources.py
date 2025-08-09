#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import random
import threading
import time

import zelos_sdk as zelos


def stream_motor() -> None:
    source = zelos.TraceSource("motor")
    telemetry = source.add_event(
        "telemetry",
        [
            zelos.TraceEventFieldMetadata("rpm", zelos.DataType.Float64, "rpm"),
            zelos.TraceEventFieldMetadata("torque", zelos.DataType.Float64, "Nm"),
        ],
    )
    while True:
        telemetry.log(
            rpm=2000.0 + random.uniform(-100.0, 100.0),
            torque=50.0 + random.uniform(-5.0, 5.0),
        )
        time.sleep(0.01)  # 100 Hz


def stream_battery() -> None:
    source = zelos.TraceSource("battery")
    status = source.add_event(
        "status",
        [
            zelos.TraceEventFieldMetadata("voltage", zelos.DataType.Float64, "V"),
            zelos.TraceEventFieldMetadata("current", zelos.DataType.Float64, "A"),
            zelos.TraceEventFieldMetadata("soc", zelos.DataType.Float64, "%"),
        ],
    )
    soc = 85.0
    while True:
        soc = max(20.0, soc - 0.1)
        status.log(
            voltage=48.0 + random.uniform(-0.5, 0.5),
            current=random.uniform(-10.0, 50.0),
            soc=soc,
        )
        time.sleep(1.0)


def stream_gps() -> None:
    source = zelos.TraceSource("gps")
    position = source.add_event(
        "position",
        [
            zelos.TraceEventFieldMetadata("lat", zelos.DataType.Float64, "deg"),
            zelos.TraceEventFieldMetadata("lon", zelos.DataType.Float64, "deg"),
            zelos.TraceEventFieldMetadata("alt", zelos.DataType.Float64, "m"),
        ],
    )
    base_lat, base_lon = 37.4419, -122.1430
    while True:
        position.log(
            lat=base_lat + random.uniform(-0.001, 0.001),
            lon=base_lon + random.uniform(-0.001, 0.001),
            alt=30.0 + random.uniform(-1.0, 1.0),
        )
        time.sleep(0.1)


def main() -> None:
    zelos.init()
    threads = [
        threading.Thread(target=stream_motor, daemon=True),
        threading.Thread(target=stream_battery, daemon=True),
        threading.Thread(target=stream_gps, daemon=True),
    ]
    for t in threads:
        t.start()
    try:
        for t in threads:
            t.join()
    except KeyboardInterrupt:
        print("Stopping streams...")


if __name__ == "__main__":
    main()
