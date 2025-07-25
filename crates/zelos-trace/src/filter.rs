use anyhow::{Result, anyhow};
use uuid::Uuid;
use zelos_trace_types::ipc::{IpcMessage, IpcMessageWithId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Filter {
    pub segment_id: Option<Uuid>,
    pub source_name: Option<String>,
    pub event_name: Option<String>,
}

impl Filter {
    pub fn new(
        segment_id: Option<Uuid>,
        source_name: Option<String>,
        event_name: Option<String>,
    ) -> Self {
        Self {
            segment_id,
            source_name,
            event_name,
        }
    }

    pub fn any() -> Self {
        Self {
            segment_id: None,
            source_name: None,
            event_name: None,
        }
    }

    pub fn parse(filter: &str) -> Result<Self> {
        // Split our filter string by `/`
        let (uuid_str, rest) = filter.split_once("/").ok_or(anyhow!("Unable to split"))?;
        let (source_name_str, event_name_str) =
            rest.split_once("/").ok_or(anyhow!("Unable to split"))?;

        let segment_id = match uuid_str {
            "*" => None,
            uuid_str => Some(Uuid::parse_str(uuid_str)?),
        };
        let source_name = match source_name_str {
            "*" => None,
            source_name => Some(source_name.to_string()),
        };
        let event_name = match event_name_str {
            "*" => None,
            event_name => Some(event_name.to_string()),
        };

        Ok(Self {
            segment_id,
            source_name,
            event_name,
        })
    }

    pub fn matches(&self, msg: &IpcMessageWithId) -> bool {
        match self.segment_id {
            Some(segment_id) if segment_id != msg.segment_id => return false,
            _ => {}
        }

        if let Some(match_source_name) = &self.source_name {
            if match_source_name != &msg.source_name {
                return false;
            }
        }

        match (&self.event_name, &msg.msg) {
            (Some(event_name), IpcMessage::TraceEvent(e)) => {
                if event_name != &e.name {
                    return false;
                }
            }
            (Some(_), _) => {
                // If message is not a TraceEvent, it can't match by event name
                return false;
            }
            _ => {}
        }

        // If we've gotten this far, the message must match
        true
    }
}
