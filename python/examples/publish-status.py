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
    source = zelos.TraceSource("status-demo")

    heartbeat = source.add_event(
        "heartbeat",
        [
            zelos.TraceEventFieldMetadata("seq", zelos.DataType.UInt64),
            zelos.TraceEventFieldMetadata("timestamp", zelos.DataType.Int64, "ns"),
        ],
    )

    # Emit a few heartbeats then exit (finite example)
    for seq in range(1, 6):
        heartbeat.log(seq=seq, timestamp=time.time_ns())
        print(f"published heartbeat seq={seq}")
        time.sleep(0.1)


if __name__ == "__main__":
    main()
