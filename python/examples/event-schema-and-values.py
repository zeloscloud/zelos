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
    source = zelos.TraceSource("state-machine")

    state_event = source.add_event(
        "state",
        [
            zelos.TraceEventFieldMetadata("current", zelos.DataType.UInt8),
            zelos.TraceEventFieldMetadata("previous", zelos.DataType.UInt8),
            zelos.TraceEventFieldMetadata("transition_time_ms", zelos.DataType.Float64),
        ],
    )

    source.add_value_table("state", "current", {0: "IDLE", 1: "INIT", 2: "RUNNING", 3: "ERROR"})
    source.add_value_table("state", "previous", {0: "IDLE", 1: "INIT", 2: "RUNNING", 3: "ERROR"})

    transitions = [(0, 1), (1, 2), (2, 3), (3, 0)]
    previous_time = time.time()

    for prev, cur in transitions:
        now = time.time()
        dt_ms = (now - previous_time) * 1000.0
        state_event.log(current=cur, previous=prev, transition_time_ms=dt_ms)
        previous_time = now
        time.sleep(0.25)


if __name__ == "__main__":
    main()
