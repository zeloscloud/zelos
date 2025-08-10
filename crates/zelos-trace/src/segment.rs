use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rpds::HashTrieMapSync;
use uuid::Uuid;
use zelos_trace_types::{
    ipc::{self, TraceEventFieldMetadata},
    PathSegment, Signal, SignalKey, Value,
};

#[derive(Debug, Clone)]
pub struct TraceEventSchemaRef<'a> {
    pub segment: &'a TraceSegment,
    pub event_schema: &'a TraceEventSchema,
}

#[derive(Debug, Clone)]
pub struct TraceEventFieldRef<'a> {
    pub segment: &'a TraceSegment,
    pub event_schema: &'a TraceEventSchema,
    pub field: &'a TraceEventField,
}

impl TraceEventFieldRef<'_> {
    pub fn as_signal(&self) -> Signal {
        let &TraceEventFieldRef {
            segment,
            event_schema,
            field,
        } = self;

        Signal {
            data_segment_id: segment.id,
            source: segment.source.clone(),
            message: event_schema.name.clone(),
            signal: field.metadata.name.clone(),
            data_type: field.metadata.data_type.clone(),
            unit: field.metadata.unit.clone(),
            value_table: if field.values.is_empty() {
                None
            } else {
                Some(
                    field
                        .values
                        .iter()
                        .filter_map(|(k, v)| k.as_number().map(|n| (n, v.clone())))
                        .collect(),
                )
            },
        }
    }
}

impl TraceEventFieldRef<'_> {
    pub fn table_key(&self) -> String {
        format!(
            "{}/{}/{}",
            self.segment.id, self.segment.source, self.event_schema.name
        )
    }
}

#[derive(Debug, Clone)]
pub struct TraceEventField {
    pub metadata: TraceEventFieldMetadata,
    pub values: HashMap<Value, String>,
}

impl TraceEventField {
    pub fn from_ipc(msg: ipc::TraceEventFieldMetadata) -> Self {
        Self {
            metadata: msg,
            values: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TraceEventSchema {
    pub name: String,
    pub fields: Vec<TraceEventField>,
}

impl TraceEventSchema {
    pub fn from_ipc(msg: ipc::TraceEventSchema) -> Self {
        Self {
            name: msg.name,
            fields: msg
                .fields
                .into_iter()
                .map(TraceEventField::from_ipc)
                .collect(),
        }
    }

    pub fn get_field(&self, field_name: &str) -> Option<&TraceEventField> {
        self.fields
            .iter()
            .find(|field| field.metadata.name == field_name)
    }

    pub fn get_field_mut(&mut self, field_name: &str) -> Option<&mut TraceEventField> {
        self.fields
            .iter_mut()
            .find(|field| field.metadata.name == field_name)
    }

    pub fn metadata(&self) -> impl Iterator<Item = &TraceEventFieldMetadata> {
        self.fields.iter().map(|field| &field.metadata)
    }
}

#[derive(Clone, Debug)]
pub struct TraceSegment {
    pub id: Uuid,
    pub source: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub schemas: HashTrieMapSync<String, TraceEventSchema>,
}

impl TraceSegment {
    /// Create an empty trace segment when we have not received a start message
    pub fn empty(id: Uuid, source_name: String) -> Self {
        Self {
            id,
            source: source_name,
            start_time: None,
            end_time: None,
            schemas: HashTrieMapSync::new_sync(),
        }
    }

    /// Create a trace segment from a start message.
    pub fn from_ipc(id: Uuid, start: &ipc::TraceSegmentStart) -> Self {
        Self {
            id,
            source: start.source_name.clone(),
            start_time: Some(DateTime::from_timestamp_nanos(start.time_ns)),
            end_time: None,
            schemas: HashTrieMapSync::new_sync(),
        }
    }

    pub fn update_mut(&mut self, msg: &ipc::IpcMessage) {
        match msg {
            ipc::IpcMessage::TraceSegmentStart(m) => {
                self.source = m.source_name.clone();

                // Update the start time if it's earlier than the existing one
                let start_time = DateTime::from_timestamp_nanos(m.time_ns);
                if let Some(existing_start_time) = self.start_time {
                    if start_time < existing_start_time {
                        self.start_time = Some(start_time);
                    }
                } else {
                    self.start_time = Some(start_time);
                }
            }
            ipc::IpcMessage::TraceSegmentEnd(m) => {
                self.end_time = Some(DateTime::from_timestamp_nanos(m.time_ns));
            }
            ipc::IpcMessage::TraceEventSchema(m) => {
                if !self.schemas.contains_key(&m.name) {
                    self.schemas
                        .insert_mut(m.name.clone(), TraceEventSchema::from_ipc(m.clone()));
                }
            }
            ipc::IpcMessage::TraceEventFieldNamedValues(m) => {
                // Update our event schema in place
                if let Some(mut event_schema) = self.schemas.get(&m.event_name).cloned() {
                    if let Some(field) = event_schema.get_field_mut(&m.field_name) {
                        field.values.extend(m.values.clone());
                    }
                    self.schemas.insert_mut(m.event_name.clone(), event_schema);
                }
            }
            ipc::IpcMessage::TraceEvent(_m) => {
                // Do nothing
            }
        }
    }

    pub fn update(&self, msg: &ipc::IpcMessage) -> Self {
        let mut new = self.clone();
        new.update_mut(msg);
        new
    }

    pub fn maybe_event<'a>(&'a self, event_name: &str) -> Option<TraceEventSchemaRef<'a>> {
        self.schemas
            .get(event_name)
            .map(|event_schema| TraceEventSchemaRef {
                segment: self,
                event_schema,
            })
    }

    pub fn field_refs(&self) -> impl Iterator<Item = TraceEventFieldRef<'_>> {
        self.schemas.values().flat_map(|event_schema| {
            event_schema.fields.iter().map(|field| TraceEventFieldRef {
                segment: self,
                event_schema,
                field,
            })
        })
    }

    pub fn field_refs_matching<'a>(
        &'a self,
        signal_keys: &[SignalKey],
    ) -> impl Iterator<Item = TraceEventFieldRef<'a>> {
        signal_keys
            .iter()
            .flat_map(|k| self.maybe_field_ref_matching(k))
    }

    pub fn maybe_field_ref_matching<'a>(
        &'a self,
        key: &SignalKey,
    ) -> Option<TraceEventFieldRef<'a>> {
        // Return early if this signal key is for a specific uuid and we don't match it
        if let PathSegment::Uuid { uuid } = key.data_segment_id {
            if uuid != self.id {
                return None;
            }
        }

        // Return early if this signal key is for a specific source and we don't match it
        if key.source != self.source {
            return None;
        }

        // Attempt to get the message, and map it to the column
        self.schemas.get(&key.message).and_then(|event_schema| {
            event_schema
                .fields
                .iter()
                .find(|k| k.metadata.name == key.signal)
                .map(|field| TraceEventFieldRef {
                    segment: self,
                    event_schema,
                    field,
                })
        })
    }

    pub fn signals(&self) -> impl Iterator<Item = Signal> {
        self.field_refs().map(|r| r.as_signal())
    }

    pub fn signals_matching(&self, signal_keys: &[SignalKey]) -> impl Iterator<Item = Signal> {
        self.field_refs_matching(signal_keys).map(|r| r.as_signal())
    }

    /// Represent this trace segment as ipc messages
    pub fn as_ipc(&self) -> Vec<ipc::IpcMessage> {
        let mut msgs = Vec::new();

        // Send start, if we have a start timestamp
        // NOTE(jbott): this is somewhat weird, should we store trace segments if we don't have a timestamp? should we fixup timestamps from the data contained within?
        if let Some(start_time_ns) = self.start_time.and_then(|t| t.timestamp_nanos_opt()) {
            let start = ipc::TraceSegmentStart {
                time_ns: start_time_ns,
                source_name: self.source.clone(),
            };
            msgs.push(start.into());
        }

        // Iterate over all schemas and send messages as required
        for (event_name, schema) in &self.schemas {
            // Send the schema
            let event_schema = ipc::TraceEventSchema {
                name: schema.name.clone(),
                fields: schema.fields.iter().map(|f| f.metadata.clone()).collect(),
            };
            msgs.push(event_schema.into());

            // For each field with values, send the hashmap
            for (field_name, values) in schema
                .fields
                .iter()
                .filter(|f| !f.values.is_empty())
                .map(|f| (f.metadata.name.clone(), f.values.clone()))
            {
                let event_field_named_values = ipc::TraceEventFieldNamedValues {
                    event_name: event_name.clone(),
                    field_name: field_name.clone(),
                    values,
                };
                msgs.push(event_field_named_values.into());
            }
        }

        // Send end, if we have an end timestamp
        if let Some(end_time_ns) = self.end_time.and_then(|t| t.timestamp_nanos_opt()) {
            let end = ipc::TraceSegmentEnd {
                time_ns: end_time_ns,
            };
            msgs.push(end.into());
        }

        msgs
    }
}
