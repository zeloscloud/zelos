//! This module provides helper functions going to / from our in-memory rust structs and the autogenerated protobuf
//! structs. This is an attempt to ensure we perform validation of incoming data in one place (here) since prost does
//! not give us options to ensure that messages exist (all are options). Hopefully this also gives us flexibility with
//! our in-memory data structures going forward.

use uuid::Uuid;
use zelos_trace_types::ipc;

use crate::error::Error;

// ===== DataType =====
impl From<zelos_trace_types::DataType> for super::DataType {
    fn from(value: zelos_trace_types::DataType) -> Self {
        match value {
            zelos_trace_types::DataType::Int8 => Self::Int8,
            zelos_trace_types::DataType::Int16 => Self::Int16,
            zelos_trace_types::DataType::Int32 => Self::Int32,
            zelos_trace_types::DataType::Int64 => Self::Int64,
            zelos_trace_types::DataType::UInt8 => Self::Uint8,
            zelos_trace_types::DataType::UInt16 => Self::Uint16,
            zelos_trace_types::DataType::UInt32 => Self::Uint32,
            zelos_trace_types::DataType::UInt64 => Self::Uint64,
            zelos_trace_types::DataType::Float32 => Self::Float32,
            zelos_trace_types::DataType::Float64 => Self::Float64,
            zelos_trace_types::DataType::String => Self::String,
            zelos_trace_types::DataType::Binary => Self::Binary,
            zelos_trace_types::DataType::Boolean => Self::Bool,
            zelos_trace_types::DataType::TimestampNs => Self::TimestampNs,
        }
    }
}
impl TryInto<zelos_trace_types::DataType> for super::DataType {
    type Error = Error;

    fn try_into(self) -> Result<zelos_trace_types::DataType, Self::Error> {
        match self {
            Self::Int8 => Ok(zelos_trace_types::DataType::Int8),
            Self::Int16 => Ok(zelos_trace_types::DataType::Int16),
            Self::Int32 => Ok(zelos_trace_types::DataType::Int32),
            Self::Int64 => Ok(zelos_trace_types::DataType::Int64),
            Self::Uint8 => Ok(zelos_trace_types::DataType::UInt8),
            Self::Uint16 => Ok(zelos_trace_types::DataType::UInt16),
            Self::Uint32 => Ok(zelos_trace_types::DataType::UInt32),
            Self::Uint64 => Ok(zelos_trace_types::DataType::UInt64),
            Self::Float32 => Ok(zelos_trace_types::DataType::Float32),
            Self::Float64 => Ok(zelos_trace_types::DataType::Float64),
            Self::String => Ok(zelos_trace_types::DataType::String),
            Self::Binary => Ok(zelos_trace_types::DataType::Binary),
            Self::Bool => Ok(zelos_trace_types::DataType::Boolean),
            Self::TimestampNs => Ok(zelos_trace_types::DataType::TimestampNs),
            Self::Unspecified => Err(Self::Error::MissingDataType),
        }
    }
}

// ===== Value =====
impl From<zelos_trace_types::Value> for super::Value {
    fn from(v: zelos_trace_types::Value) -> Self {
        Self {
            value: Some(match v {
                zelos_trace_types::Value::Int8(v) => super::value::Value::Int8(v.into()),
                zelos_trace_types::Value::Int16(v) => super::value::Value::Int16(v.into()),
                zelos_trace_types::Value::Int32(v) => super::value::Value::Int32(v.into()),
                zelos_trace_types::Value::Int64(v) => super::value::Value::Int64(v),
                zelos_trace_types::Value::UInt8(v) => super::value::Value::Uint8(v.into()),
                zelos_trace_types::Value::UInt16(v) => super::value::Value::Uint16(v.into()),
                zelos_trace_types::Value::UInt32(v) => super::value::Value::Uint32(v.into()),
                zelos_trace_types::Value::UInt64(v) => super::value::Value::Uint64(v),
                zelos_trace_types::Value::Float32(v) => super::value::Value::Float32(v),
                zelos_trace_types::Value::Float64(v) => super::value::Value::Float64(v),
                zelos_trace_types::Value::TimestampNs(v) => super::value::Value::TimestampNs(v),
                zelos_trace_types::Value::Binary(v) => super::value::Value::Binary(v),
                zelos_trace_types::Value::String(v) => super::value::Value::String(v),
                zelos_trace_types::Value::Boolean(v) => super::value::Value::Bool(v),
            }),
        }
    }
}
impl TryInto<zelos_trace_types::Value> for super::Value {
    type Error = Error;

    fn try_into(self) -> Result<zelos_trace_types::Value, Self::Error> {
        let value = self.value.ok_or(Error::MissingValue)?;

        Ok(match value {
            super::value::Value::Int8(v) => zelos_trace_types::Value::Int8(v.try_into()?),
            super::value::Value::Int16(v) => zelos_trace_types::Value::Int16(v.try_into()?),
            super::value::Value::Int32(v) => zelos_trace_types::Value::Int32(v.try_into()?),
            super::value::Value::Int64(v) => zelos_trace_types::Value::Int64(v),
            super::value::Value::Uint8(v) => zelos_trace_types::Value::UInt8(v.try_into()?),
            super::value::Value::Uint16(v) => zelos_trace_types::Value::UInt16(v.try_into()?),
            super::value::Value::Uint32(v) => zelos_trace_types::Value::UInt32(v.try_into()?),
            super::value::Value::Uint64(v) => zelos_trace_types::Value::UInt64(v),
            super::value::Value::Float32(v) => zelos_trace_types::Value::Float32(v),
            super::value::Value::Float64(v) => zelos_trace_types::Value::Float64(v),
            super::value::Value::TimestampNs(v) => zelos_trace_types::Value::TimestampNs(v),
            super::value::Value::Binary(v) => zelos_trace_types::Value::Binary(v),
            super::value::Value::String(v) => zelos_trace_types::Value::String(v),
            super::value::Value::Bool(v) => zelos_trace_types::Value::Boolean(v),
        })
    }
}

// ===== TraceSegmentStart =====
impl From<ipc::TraceSegmentStart> for super::TraceSegmentStart {
    fn from(value: ipc::TraceSegmentStart) -> Self {
        Self {
            time_ns: value.time_ns,
            source_name: value.source_name,
        }
    }
}

// ===== TraceSegmentEnd =====
impl From<ipc::TraceSegmentEnd> for super::TraceSegmentEnd {
    fn from(value: ipc::TraceSegmentEnd) -> Self {
        Self {
            time_ns: value.time_ns,
        }
    }
}

// ===== TraceEventFieldMetadata =====
impl From<ipc::TraceEventFieldMetadata> for super::TraceEventFieldMetadata {
    fn from(value: ipc::TraceEventFieldMetadata) -> Self {
        let data_type: super::DataType = value.data_type.into();
        Self {
            name: value.name,
            data_type: data_type.into(),
            unit: value.unit,
        }
    }
}
impl TryInto<ipc::TraceEventFieldMetadata> for super::TraceEventFieldMetadata {
    type Error = Error;

    fn try_into(self) -> Result<ipc::TraceEventFieldMetadata, Self::Error> {
        let data_type = self.data_type().try_into()?;
        Ok(ipc::TraceEventFieldMetadata {
            name: self.name,
            data_type,
            unit: self.unit,
        })
    }
}

// ===== TraceEventSchema =====
impl From<ipc::TraceEventSchema> for super::TraceEventSchema {
    fn from(value: ipc::TraceEventSchema) -> Self {
        Self {
            name: value.name,
            fields: value.fields.into_iter().map(|field| field.into()).collect(),
        }
    }
}
impl TryInto<ipc::TraceEventSchema> for super::TraceEventSchema {
    type Error = Error;

    fn try_into(self) -> Result<ipc::TraceEventSchema, Self::Error> {
        let fields = self
            .fields
            .into_iter()
            .map(|field| field.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ipc::TraceEventSchema {
            name: self.name,
            fields,
        })
    }
}

// ===== Helpers for Vec<TraceEventFieldEntry> =====
impl From<(String, zelos_trace_types::Value)> for super::TraceEventFieldEntry {
    fn from((name, value): (String, zelos_trace_types::Value)) -> Self {
        Self {
            name,
            value: Some(value.into()),
        }
    }
}
impl TryInto<(String, zelos_trace_types::Value)> for super::TraceEventFieldEntry {
    type Error = Error;

    fn try_into(self) -> Result<(String, zelos_trace_types::Value), Self::Error> {
        Ok((
            self.name,
            self.value.ok_or(Error::MissingValue)?.try_into()?,
        ))
    }
}

// ===== TraceEvent =====
impl From<ipc::TraceEvent> for super::TraceEvent {
    fn from(value: ipc::TraceEvent) -> Self {
        Self {
            time_ns: value.time_ns,
            name: value.name,
            fields: value.fields.into_iter().map(|field| field.into()).collect(),
        }
    }
}
impl TryInto<ipc::TraceEvent> for super::TraceEvent {
    type Error = Error;

    fn try_into(self) -> Result<ipc::TraceEvent, Self::Error> {
        // TODO(jbott): error on duplicate keys?
        let fields: Vec<(String, zelos_trace_types::Value)> = self
            .fields
            .into_iter()
            .map(|field| field.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ipc::TraceEvent {
            time_ns: self.time_ns,
            name: self.name,
            fields: fields.into_iter().collect(),
        })
    }
}

// ===== Helpers for Vec<TraceEventFieldNamedValuesEntry> =====
impl From<(zelos_trace_types::Value, String)> for super::TraceEventFieldNamedValuesEntry {
    fn from((value, name): (zelos_trace_types::Value, String)) -> Self {
        Self {
            name,
            value: Some(value.into()),
        }
    }
}
impl TryInto<(zelos_trace_types::Value, String)> for super::TraceEventFieldNamedValuesEntry {
    type Error = Error;

    fn try_into(self) -> Result<(zelos_trace_types::Value, String), Self::Error> {
        Ok((
            self.value.ok_or(Error::MissingValue)?.try_into()?,
            self.name,
        ))
    }
}

// ===== TraceEventFieldNamedValues =====
impl From<ipc::TraceEventFieldNamedValues> for super::TraceEventFieldNamedValues {
    fn from(value: ipc::TraceEventFieldNamedValues) -> Self {
        Self {
            event_name: value.event_name,
            field_name: value.field_name,
            values: value.values.into_iter().map(|value| value.into()).collect(),
        }
    }
}
impl TryInto<ipc::TraceEventFieldNamedValues> for super::TraceEventFieldNamedValues {
    type Error = Error;

    fn try_into(self) -> Result<ipc::TraceEventFieldNamedValues, Self::Error> {
        // TODO(jbott): error on duplicate keys?
        let values: Vec<(zelos_trace_types::Value, String)> = self
            .values
            .into_iter()
            .map(|value| value.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ipc::TraceEventFieldNamedValues {
            event_name: self.event_name,
            field_name: self.field_name,
            values: values.into_iter().collect(),
        })
    }
}

// ===== TraceMessage =====
impl From<ipc::IpcMessageWithId> for super::TraceMessage {
    fn from(value: ipc::IpcMessageWithId) -> Self {
        Self {
            segment_id: value.segment_id.into_bytes().to_vec(),
            source_name: value.source_name,
            msg: Some(match value.msg {
                ipc::IpcMessage::TraceEvent(event) => {
                    super::trace_message::Msg::Event(event.into())
                }
                ipc::IpcMessage::TraceSegmentStart(segment_start) => {
                    super::trace_message::Msg::SegmentStart(segment_start.into())
                }
                ipc::IpcMessage::TraceSegmentEnd(segment_end) => {
                    super::trace_message::Msg::SegmentEnd(segment_end.into())
                }
                ipc::IpcMessage::TraceEventSchema(schema) => {
                    super::trace_message::Msg::EventSchema(schema.into())
                }
                ipc::IpcMessage::TraceEventFieldNamedValues(field_named_values) => {
                    super::trace_message::Msg::EventFieldNamedValues(field_named_values.into())
                }
            }),
        }
    }
}
impl TryInto<ipc::IpcMessageWithId> for super::TraceMessage {
    type Error = Error;

    fn try_into(self) -> Result<ipc::IpcMessageWithId, Self::Error> {
        let segment_id = Uuid::from_slice(&self.segment_id)?;
        let source_name = self.source_name;
        let msg = match self.msg.ok_or(Error::MissingMessage)? {
            super::trace_message::Msg::SegmentStart(trace_segment_start) => {
                ipc::IpcMessage::TraceSegmentStart(trace_segment_start.into())
            }
            super::trace_message::Msg::SegmentEnd(trace_segment_end) => {
                ipc::IpcMessage::TraceSegmentEnd(trace_segment_end.into())
            }
            super::trace_message::Msg::EventSchema(trace_event_schema) => {
                ipc::IpcMessage::TraceEventSchema(trace_event_schema.try_into()?)
            }
            super::trace_message::Msg::EventFieldNamedValues(trace_event_field_named_values) => {
                ipc::IpcMessage::TraceEventFieldNamedValues(
                    trace_event_field_named_values.try_into()?,
                )
            }
            super::trace_message::Msg::Event(trace_event) => {
                ipc::IpcMessage::TraceEvent(trace_event.try_into()?)
            }
        };
        Ok(ipc::IpcMessageWithId {
            segment_id,
            source_name,
            msg,
        })
    }
}

impl From<super::TraceSegmentStart> for ipc::TraceSegmentStart {
    fn from(val: super::TraceSegmentStart) -> Self {
        ipc::TraceSegmentStart {
            time_ns: val.time_ns,
            source_name: val.source_name,
        }
    }
}

impl From<super::TraceSegmentEnd> for ipc::TraceSegmentEnd {
    fn from(val: super::TraceSegmentEnd) -> Self {
        ipc::TraceSegmentEnd {
            time_ns: val.time_ns,
        }
    }
}
