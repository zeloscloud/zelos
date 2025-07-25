use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use uuid::Uuid;
use zelos_trace_types::{
    Value,
    ipc::{
        IpcMessage, IpcMessageWithId, Sender, TraceEvent, TraceEventFieldMetadata,
        TraceEventFieldNamedValues, TraceEventSchema, TraceSegmentEnd, TraceSegmentStart,
    },
};

use crate::time::now_time_ns;

/// TraceSourceEvent is a child of a TraceSource, which contains the schema for the event, as well as helpers for building and emitting new events for that schema.
#[derive(Debug)]
pub struct TraceSourceEvent {
    id: Uuid,
    source_name: String,
    sender: Sender,
    pub name: String,
    pub schema: Vec<TraceEventFieldMetadata>,
}

impl TraceSourceEvent {
    pub fn emit(&self, time_ns: i64, fields: impl Iterator<Item = (String, Value)>) -> Result<()> {
        let evt = TraceEvent {
            time_ns,
            name: self.name.clone(),
            fields: fields.collect(),
        };

        self.sender.send(IpcMessageWithId {
            segment_id: self.id,
            source_name: self.source_name.clone(),
            msg: IpcMessage::TraceEvent(evt),
        })?;

        Ok(())
    }

    pub async fn emit_async(
        &self,
        time_ns: i64,
        fields: impl Iterator<Item = (String, Value)>,
    ) -> Result<()> {
        let evt = TraceEvent {
            time_ns,
            name: self.name.clone(),
            fields: fields.collect(),
        };

        self.sender
            .send_async(IpcMessageWithId {
                segment_id: self.id,
                source_name: self.source_name.clone(),
                msg: IpcMessage::TraceEvent(evt),
            })
            .await?;

        Ok(())
    }

    pub fn build(&self) -> builder::EventBuilder<'_> {
        builder::EventBuilder::new(self)
    }
}

/// TraceSource is the main interface to emitting Zelos trace events. It provides convencience methods for building new
/// event schemas and emitting events from them.
#[derive(Debug)]
pub struct TraceSource {
    pub id: Uuid,
    pub source_name: String,
    sender: Sender,
    events: RwLock<HashMap<String, Arc<TraceSourceEvent>>>,
}

impl TraceSource {
    pub fn new(source_name: &str, sender: Sender) -> Self {
        let id = Uuid::now_v7();
        let src = TraceSource {
            id,
            source_name: source_name.to_string(),
            sender,
            events: RwLock::new(HashMap::new()),
        };

        tracing::debug!(?id, ?source_name, "TraceSource::new");

        if let Err(e) = src.emit_start() {
            tracing::error!("Error emitting trace segment start: {}", e);
        }

        src
    }

    fn emit(&self, msg: IpcMessage) -> Result<()> {
        self.sender.send(IpcMessageWithId {
            segment_id: self.id,
            source_name: self.source_name.clone(),
            msg,
        })?;

        Ok(())
    }

    async fn emit_async(&self, msg: IpcMessage) -> Result<()> {
        self.sender
            .send_async(IpcMessageWithId {
                segment_id: self.id,
                source_name: self.source_name.clone(),
                msg,
            })
            .await?;

        Ok(())
    }

    pub fn emit_start(&self) -> Result<()> {
        self.emit(IpcMessage::TraceSegmentStart(TraceSegmentStart {
            time_ns: now_time_ns(),
            source_name: self.source_name.clone(),
        }))
    }

    pub fn emit_end(&self) -> Result<()> {
        self.emit(IpcMessage::TraceSegmentEnd(TraceSegmentEnd {
            time_ns: now_time_ns(),
        }))
    }

    pub fn add_value_table(
        &self,
        name: &str,
        field_name: &str,
        values: impl Iterator<Item = (Value, String)>,
    ) -> Result<()> {
        self.emit(IpcMessage::TraceEventFieldNamedValues(
            TraceEventFieldNamedValues {
                event_name: name.to_string(),
                field_name: field_name.to_string(),
                values: values.collect(),
            },
        ))
    }

    pub fn add_event(
        &self,
        name: &str,
        schema: impl Iterator<Item = TraceEventFieldMetadata>,
    ) -> Result<Arc<TraceSourceEvent>> {
        if self.events.read().contains_key(name) {
            return Err(anyhow!("Event={} already exists", name));
        }

        let msg = Arc::new(TraceSourceEvent {
            id: self.id,
            source_name: self.source_name.clone(),
            sender: self.sender.clone(),
            name: name.to_string(),
            schema: schema.collect(),
        });

        // Emit the event to the router
        self.emit(IpcMessage::TraceEventSchema(TraceEventSchema {
            name: name.to_string(),
            fields: msg.schema.clone(),
        }))?;

        // Insert the event into our metadata store
        self.events.write().insert(name.to_string(), msg.clone());

        Ok(msg)
    }

    pub async fn add_event_async(
        &self,
        name: &str,
        schema: impl Iterator<Item = TraceEventFieldMetadata>,
    ) -> Result<Arc<TraceSourceEvent>> {
        if self.events.read().contains_key(name) {
            return Err(anyhow!("Event={} already exists", name));
        }

        let msg = Arc::new(TraceSourceEvent {
            id: self.id,
            source_name: self.source_name.clone(),
            sender: self.sender.clone(),
            name: name.to_string(),
            schema: schema.collect(),
        });

        // Emit the event to the router
        self.emit_async(IpcMessage::TraceEventSchema(TraceEventSchema {
            name: name.to_string(),
            fields: msg.schema.clone(),
        }))
        .await?;

        // Insert the event into our metadata store
        self.events.write().insert(name.to_string(), msg.clone());

        Ok(msg)
    }

    pub fn get_event(&self, name: &str) -> Result<Arc<TraceSourceEvent>> {
        self.events
            .read()
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Event not found"))
    }

    pub fn build_event<'a>(&'a self, name: &'a str) -> builder::TraceSourceEventBuilder<'a> {
        builder::TraceSourceEventBuilder::new(self, name)
    }
}

impl Drop for TraceSource {
    fn drop(&mut self) {
        if let Err(e) = self.emit_end() {
            tracing::debug!("Error emitting trace segment end: {}", e);
        }
    }
}

pub mod builder {
    use zelos_trace_types::DataType;

    use super::*;

    #[must_use]
    pub struct EventBuilder<'a> {
        parent: &'a TraceSourceEvent,
        data: HashMap<String, Value>,
    }

    impl<'a> EventBuilder<'a> {
        pub(crate) fn new(parent: &'a TraceSourceEvent) -> Self {
            EventBuilder {
                parent,
                data: HashMap::new(),
            }
        }

        /// Emit the event at the current time.
        pub fn emit(mut self) -> Result<()> {
            self.parent.emit(now_time_ns(), self.data.drain())
        }

        /// Emit the event at a specific time.
        pub fn emit_at(mut self, time_ns: i64) -> Result<()> {
            self.parent.emit(time_ns, self.data.drain())
        }

        /// Emit the event at the current time via async
        pub async fn emit_async(mut self) -> Result<()> {
            self.parent
                .emit_async(now_time_ns(), self.data.drain())
                .await
        }

        /// Emit the event at a specific time via async
        pub async fn emit_at_async(mut self, time_ns: i64) -> Result<()> {
            self.parent.emit_async(time_ns, self.data.drain()).await
        }

        /// Attempt to insert a value into the event, returning an error if the field is not found or the type does not match.
        pub fn try_insert(&mut self, name: &str, value: Value) -> Result<()> {
            // Find the field in the schema
            let field = self
                .parent
                .schema
                .iter()
                .find(|field| field.name == name)
                .ok_or_else(|| anyhow!("Field '{}' not found in schema", name))?;

            // Check if our value matches the field type
            if field.data_type != value.data_type() {
                return Err(anyhow!(
                    "Type mismatch for field '{}': expected {:?}, found {:?}",
                    name,
                    field.data_type,
                    value.data_type()
                ));
            }

            // Insert the value into our event
            self.data.insert(name.to_string(), value);
            Ok(())
        }

        pub fn try_insert_i8(mut self, name: &str, value: i8) -> Result<Self> {
            self.try_insert(name, Value::Int8(value))?;
            Ok(self)
        }

        pub fn try_insert_i16(mut self, name: &str, value: i16) -> Result<Self> {
            self.try_insert(name, Value::Int16(value))?;
            Ok(self)
        }

        pub fn try_insert_i32(mut self, name: &str, value: i32) -> Result<Self> {
            self.try_insert(name, Value::Int32(value))?;
            Ok(self)
        }

        pub fn try_insert_i64(mut self, name: &str, value: i64) -> Result<Self> {
            self.try_insert(name, Value::Int64(value))?;
            Ok(self)
        }

        pub fn try_insert_u8(mut self, name: &str, value: u8) -> Result<Self> {
            self.try_insert(name, Value::UInt8(value))?;
            Ok(self)
        }

        pub fn try_insert_u16(mut self, name: &str, value: u16) -> Result<Self> {
            self.try_insert(name, Value::UInt16(value))?;
            Ok(self)
        }

        pub fn try_insert_u32(mut self, name: &str, value: u32) -> Result<Self> {
            self.try_insert(name, Value::UInt32(value))?;
            Ok(self)
        }

        pub fn try_insert_u64(mut self, name: &str, value: u64) -> Result<Self> {
            self.try_insert(name, Value::UInt64(value))?;
            Ok(self)
        }

        pub fn try_insert_f32(mut self, name: &str, value: f32) -> Result<Self> {
            self.try_insert(name, Value::Float32(value))?;
            Ok(self)
        }

        pub fn try_insert_f64(mut self, name: &str, value: f64) -> Result<Self> {
            self.try_insert(name, Value::Float64(value))?;
            Ok(self)
        }

        pub fn try_insert_timestamp_ns(mut self, name: &str, value: i64) -> Result<Self> {
            self.try_insert(name, Value::TimestampNs(value))?;
            Ok(self)
        }

        pub fn try_insert_binary(mut self, name: &str, value: Vec<u8>) -> Result<Self> {
            self.try_insert(name, Value::Binary(value))?;
            Ok(self)
        }

        pub fn try_insert_string(mut self, name: &str, value: String) -> Result<Self> {
            self.try_insert(name, Value::String(value))?;
            Ok(self)
        }

        pub fn try_insert_bool(mut self, name: &str, value: bool) -> Result<Self> {
            self.try_insert(name, Value::Boolean(value))?;
            Ok(self)
        }
    }

    #[must_use]
    pub struct TraceSourceEventBuilder<'a> {
        source: &'a TraceSource,
        name: &'a str,
        schema: HashMap<String, TraceEventFieldMetadata>,
    }

    impl<'a> TraceSourceEventBuilder<'a> {
        pub(crate) fn new(source: &'a TraceSource, name: &'a str) -> Self {
            TraceSourceEventBuilder {
                source,
                name,
                schema: HashMap::new(),
            }
        }

        /// Build the event and add it to the source.
        pub fn build(self) -> Result<Arc<TraceSourceEvent>> {
            self.source.add_event(self.name, self.schema.into_values())
        }

        /// Build the event and add it to the source via async.
        pub async fn build_async(self) -> Result<Arc<TraceSourceEvent>> {
            self.source
                .add_event_async(self.name, self.schema.into_values())
                .await
        }

        pub fn add_field(mut self, name: &str, data_type: DataType, unit: Option<String>) -> Self {
            self.schema.insert(
                name.to_string(),
                TraceEventFieldMetadata {
                    name: name.to_string(),
                    data_type,
                    unit,
                },
            );
            self
        }

        pub fn add_i8_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Int8, unit)
        }

        pub fn add_i16_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Int16, unit)
        }

        pub fn add_i32_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Int32, unit)
        }

        pub fn add_i64_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Int64, unit)
        }

        pub fn add_u8_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::UInt8, unit)
        }

        pub fn add_u16_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::UInt16, unit)
        }

        pub fn add_u32_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::UInt32, unit)
        }

        pub fn add_u64_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::UInt64, unit)
        }

        pub fn add_f32_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Float32, unit)
        }

        pub fn add_f64_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Float64, unit)
        }

        pub fn add_timestamp_ns_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::TimestampNs, unit)
        }

        pub fn add_binary_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Binary, unit)
        }

        pub fn add_string_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::String, unit)
        }

        pub fn add_bool_field(self, name: &str, unit: Option<String>) -> Self {
            self.add_field(name, DataType::Boolean, unit)
        }
    }
}

#[cfg(test)]
mod test {
    use zelos_trace_types::DataType;

    use super::*;

    #[test]
    fn test_source_basic() -> Result<()> {
        // Create our flume channel for messages
        let (sender, receiver) = flume::unbounded::<IpcMessageWithId>();
        // Create our source
        let src = TraceSource::new("src", sender.clone());
        let id = src.id.clone();

        // Check we get a start event on creation
        {
            let m = receiver.recv()?;
            assert_eq!(m.segment_id, id);
            assert!(matches!(m.msg, IpcMessage::TraceSegmentStart(_)));
        }

        // Create our event
        let evt = src
            .build_event("hello")
            .add_i32_field("sig", None)
            .build()?;

        // Check specific contents of the TraceEventSchema
        {
            let m = receiver.recv()?;
            assert_eq!(m.segment_id, id);
            if let IpcMessage::TraceEventSchema(schema) = &m.msg {
                assert_eq!(schema.name, "hello");
                assert_eq!(schema.fields.len(), 1);
                assert_eq!(schema.fields[0].name, "sig");
                assert_eq!(schema.fields[0].data_type, DataType::Int32);
            } else {
                panic!("Expected TraceEventSchema");
            }
        }

        // Use our event handle to insert
        evt.build().try_insert_i32("sig", 10)?.emit()?;

        // Check specific contents of the first TraceEvent
        {
            let m = receiver.recv()?;
            assert_eq!(m.segment_id, id);
            if let IpcMessage::TraceEvent(event) = &m.msg {
                assert_eq!(event.name, "hello");
                assert_eq!(event.fields.len(), 1);
                let data = event.fields.iter().collect::<Vec<_>>();
                assert_eq!(data[0].0, "sig");
                assert_eq!(*data[0].1, Value::Int32(10));
            } else {
                panic!("Expected TraceEvent");
            }
        }

        // Get the message and insert using that
        src.get_event("hello")?
            .build()
            .try_insert_i32("sig", 20)?
            .emit()?;

        // Check specific contents of the second TraceEvent
        {
            let m = receiver.recv()?;
            assert_eq!(m.segment_id, id);
            if let IpcMessage::TraceEvent(event) = &m.msg {
                assert_eq!(event.name, "hello");
                assert_eq!(event.fields.len(), 1);
                let data = event.fields.iter().collect::<Vec<_>>();
                assert_eq!(data[0].0, "sig");
                assert_eq!(*data[0].1, Value::Int32(20));
            } else {
                panic!("Expected TraceEvent");
            }
        }

        // Drop the source to trigger the end event
        drop(src);

        // Check we get an end event on drop
        {
            let m = receiver.recv()?;
            assert_eq!(m.segment_id, id);
            assert!(matches!(m.msg, IpcMessage::TraceSegmentEnd(_)));
        }

        Ok(())
    }
}
