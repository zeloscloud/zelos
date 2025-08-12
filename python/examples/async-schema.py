#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("async-schema")

    # Register schema and emit
    evt = source.add_event(
        "measurement",
        [zelos.TraceEventFieldMetadata("value", zelos.DataType.Float64)],
    )

    for i in range(3):
        evt.log(value=float(i))


if __name__ == "__main__":
    main()
