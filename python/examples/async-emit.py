#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import threading
import time

import zelos_sdk as zelos


def emit_in_background(event: "zelos.TraceEvent") -> None:
    for i in range(5):
        event.log(value=float(i))
        time.sleep(0.1)


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("async-demo")

    event = source.add_event(
        "async_values",
        [zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64)],
    )

    thread = threading.Thread(target=emit_in_background, args=(event,), daemon=True)
    thread.start()

    thread.join()


if __name__ == "__main__":
    main()
