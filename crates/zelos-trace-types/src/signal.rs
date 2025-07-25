use std::collections::HashMap;

#[cfg(feature = "duckdb")]
use anyhow::{Result, anyhow};
#[cfg(feature = "duckdb")]
use duckdb::Row;
use serde::Serialize;
use serde_json::Number;
#[cfg(feature = "ts-rs")]
use ts_rs::TS;
use uuid::Uuid;

use crate::{DataType, SignalKey};

#[cfg(feature = "duckdb")]
fn parse_uuid_from_db(value: duckdb::types::Value) -> Result<Uuid> {
    match value {
        duckdb::types::Value::Text(s) => {
            Uuid::parse_str(&s).map_err(|_| anyhow!("Could not parse UUID"))
        }
        _ => return Err(anyhow!("Could not get UUID")),
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
pub struct Signal {
    pub data_segment_id: Uuid,
    pub source: String,
    pub message: String,
    pub signal: String,
    pub data_type: DataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_table: Option<HashMap<Number, String>>,
}

impl Signal {
    pub fn key(&self) -> SignalKey {
        SignalKey {
            data_segment_id: crate::PathSegment::Uuid {
                uuid: self.data_segment_id,
            },
            source: self.source.clone(),
            message: self.message.clone(),
            signal: self.signal.clone(),
        }
    }

    pub fn key_string(&self) -> String {
        format!(
            "{}/{}/{}.{}",
            self.data_segment_id, self.source, self.message, self.signal
        )
    }

    pub fn fully_qualified_table_name(&self) -> String {
        format!(
            r#""{}"."{}/{}""#,
            self.data_segment_id, self.source, self.message
        )
    }

    #[cfg(feature = "duckdb")]
    pub fn from_row(row: &Row, value_table: Option<&HashMap<Number, String>>) -> Result<Signal> {
        Ok(Signal {
            data_segment_id: parse_uuid_from_db(row.get("data_segment_id")?)?,
            source: row.get("source")?,
            message: row.get("message")?,
            signal: row.get("signal")?,
            unit: row.get("unit")?,
            data_type: row
                .get::<&str, String>("data_type")
                .map_err(|e| e.into())
                .and_then(|s| DataType::from_duckdb_type(&s))?,
            value_table: value_table.cloned(),
        })
    }
}
