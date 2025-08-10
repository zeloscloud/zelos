#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = [ "zelos-sdk" ]
# ///
from __future__ import annotations

from typing import Any, Dict
import time

import zelos_sdk as zelos


def main() -> None:
    zelos.init()
    source = zelos.TraceSource("types-demo")

    all_types = source.add_event(
        "all_types",
        [
            zelos.TraceEventFieldMetadata("bool", zelos.DataType.Boolean),
            zelos.TraceEventFieldMetadata("i8", zelos.DataType.Int8),
            zelos.TraceEventFieldMetadata("i16", zelos.DataType.Int16),
            zelos.TraceEventFieldMetadata("i32", zelos.DataType.Int32),
            zelos.TraceEventFieldMetadata("i64", zelos.DataType.Int64),
            zelos.TraceEventFieldMetadata("u8", zelos.DataType.UInt8),
            zelos.TraceEventFieldMetadata("u16", zelos.DataType.UInt16),
            zelos.TraceEventFieldMetadata("u32", zelos.DataType.UInt32),
            zelos.TraceEventFieldMetadata("u64", zelos.DataType.UInt64),
            zelos.TraceEventFieldMetadata("f32", zelos.DataType.Float32),
            zelos.TraceEventFieldMetadata("f64", zelos.DataType.Float64),
            zelos.TraceEventFieldMetadata("text", zelos.DataType.String),
            zelos.TraceEventFieldMetadata("timestamp", zelos.DataType.Int64, "ns"),
            zelos.TraceEventFieldMetadata("payload", zelos.DataType.Binary),
        ],
    )

    payload: bytes = bytes([0, 1, 2, 3, 254, 255])
    values: Dict[str, Any] = {
        "bool": True,
        "i8": -8,
        "i16": -16,
        "i32": -32,
        "i64": -64,
        "u8": 8,
        "u16": 16,
        "u32": 32,
        "u64": 64,
        "f32": 3.14,
        "f64": 6.28,
        "text": "hello",
        "timestamp": int(time.time_ns()),
        "payload": payload,
    }

    all_types.log(**values)


if __name__ == "__main__":
    main()
