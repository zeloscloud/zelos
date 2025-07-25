// Trace-related messages that are sent between processes (or inside the same process).

use std::collections::HashMap;

use derive_more::From;
use uuid::Uuid;

use crate::{DataType, Value};

#[derive(Debug, Clone)]
pub struct TraceSegmentStart {
    pub time_ns: i64,
    pub source_name: String,
}

#[derive(Debug, Clone)]
pub struct TraceSegmentEnd {
    pub time_ns: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraceEventFieldMetadata {
    pub name: String,
    pub data_type: DataType,
    pub unit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TraceEventSchema {
    pub name: String,
    pub fields: Vec<TraceEventFieldMetadata>,
}

#[derive(Debug, Clone)]
pub struct TraceEventFieldNamedValues {
    pub event_name: String,
    pub field_name: String,
    pub values: HashMap<Value, String>,
}

#[derive(Debug, Clone)]
pub struct TraceEvent {
    pub time_ns: i64,
    pub name: String,
    pub fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, From)]
pub enum IpcMessage {
    TraceSegmentStart(TraceSegmentStart),
    TraceSegmentEnd(TraceSegmentEnd),
    TraceEventSchema(TraceEventSchema),
    TraceEventFieldNamedValues(TraceEventFieldNamedValues),
    TraceEvent(TraceEvent),
}

#[derive(Debug, Clone)]
pub struct IpcMessageWithId {
    pub segment_id: Uuid,
    pub source_name: String,
    pub msg: IpcMessage,
}

pub type Sender = flume::Sender<IpcMessageWithId>;
pub type Receiver = flume::Receiver<IpcMessageWithId>;
