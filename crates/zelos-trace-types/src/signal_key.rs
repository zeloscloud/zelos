use anyhow::{anyhow, Result};
use lazy_regex::regex;
use uuid::Uuid;

use super::Signal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    Wildcard,
    Uuid { uuid: Uuid },
}

/// A signal key that uniquely identifies a signal
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SignalKey {
    pub data_segment_id: PathSegment,
    pub source: String,
    pub message: String,
    pub signal: String,
}

impl SignalKey {
    pub fn try_parse(value: &str) -> Result<SignalKey> {
        // Split all signals into their component parts
        let r = regex!(r#"^([\*\w-]+)/([^/\.]+)/([^\.]+)\.(.+)$"#);
        let parsed = r
            .captures(value)
            .ok_or_else(|| anyhow!("Could not parse signal key: {}", value))?;

        // Map the data_segment id to either some UUID or none
        let data_segment_id = match &parsed[1] {
            "*" => PathSegment::Wildcard,
            uuid => PathSegment::Uuid {
                uuid: Uuid::parse_str(uuid)?,
            },
        };
        Ok(SignalKey {
            data_segment_id,
            source: parsed[2].to_string(),
            message: parsed[3].to_string(),
            signal: parsed[4].to_string(),
        })
    }

    pub fn matches(&self, signal: &Signal) -> bool {
        let data_segment_id_matches = match self.data_segment_id {
            PathSegment::Wildcard => true,
            PathSegment::Uuid { uuid } => uuid == signal.data_segment_id,
        };

        data_segment_id_matches
            && self.source == signal.source
            && self.message == signal.message
            && self.signal == signal.signal
    }
}
