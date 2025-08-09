#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import enum
import time

import zelos_sdk as zelos


class State(enum.IntEnum):
    IDLE = 0
    INIT = 1
    RUNNING = 2
    ERROR = 3


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("controller")

    state_event = source.add_event(
        "state",
        [
            zelos.TraceEventFieldMetadata("current", zelos.DataType.UInt8),
            zelos.TraceEventFieldMetadata("previous", zelos.DataType.UInt8),
            zelos.TraceEventFieldMetadata("transition_time_ms", zelos.DataType.Float64),
        ],
    )

    source.add_value_table("state", "current", {s.value: s.name for s in State})
    source.add_value_table("state", "previous", {s.value: s.name for s in State})

    prev = State.IDLE
    prev_time = time.time()

    for cur in [State.INIT, State.RUNNING, State.ERROR, State.IDLE]:
        now = time.time()
        dt_ms = (now - prev_time) * 1000.0
        state_event.log(current=int(cur), previous=int(prev), transition_time_ms=dt_ms)
        prev, prev_time = cur, now
        time.sleep(0.5)


if __name__ == "__main__":
    main()
