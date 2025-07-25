#[cfg(feature = "duckdb")]
use anyhow::{Result, anyhow};
#[cfg(feature = "duckdb")]
use base64::prelude::*;
#[cfg(feature = "duckdb")]
use chrono::DateTime;
#[cfg(feature = "datafusion")]
use datafusion::arrow::datatypes::TimeUnit as ArrowTimeUnit;
#[cfg(feature = "datafusion")]
use datafusion::common::arrow::datatypes::DataType as ArrowDataType;
#[cfg(feature = "duckdb")]
use duckdb::ToSql;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
#[cfg(feature = "duckdb")]
use serde_json::Value;
#[cfg(feature = "ts-rs")]
use ts_rs::TS;

// Hash is required for the Python bindings
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize_enum_str, Serialize_enum_str)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts-rs", derive(TS))]
pub enum DataType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    #[serde(alias = "float")]
    Float32,
    #[serde(alias = "double")]
    Float64,
    #[serde(rename = "timestamp[ns]")]
    TimestampNs,
    /// Binary, as base64-encoded string
    Binary,
    String,
    #[serde(rename = "bool")]
    Boolean,
}

impl DataType {
    pub fn is_numeric(&self) -> bool {
        match self {
            DataType::Int8 => true,
            DataType::Int16 => true,
            DataType::Int32 => true,
            DataType::Int64 => true,
            DataType::UInt8 => true,
            DataType::UInt16 => true,
            DataType::UInt32 => true,
            DataType::UInt64 => true,
            DataType::Float32 => true,
            DataType::Float64 => true,
            DataType::TimestampNs => false,
            DataType::Binary => false,
            DataType::String => false,
            DataType::Boolean => true,
        }
    }
}

#[cfg(feature = "datafusion")]
impl DataType {
    pub fn as_arrow(&self) -> ArrowDataType {
        match self {
            DataType::Int8 => ArrowDataType::Int8,
            DataType::Int16 => ArrowDataType::Int16,
            DataType::Int32 => ArrowDataType::Int32,
            DataType::Int64 => ArrowDataType::Int64,
            DataType::UInt8 => ArrowDataType::UInt8,
            DataType::UInt16 => ArrowDataType::UInt16,
            DataType::UInt32 => ArrowDataType::UInt32,
            DataType::UInt64 => ArrowDataType::UInt64,
            DataType::Float32 => ArrowDataType::Float32,
            DataType::Float64 => ArrowDataType::Float64,
            DataType::TimestampNs => {
                ArrowDataType::Timestamp(ArrowTimeUnit::Nanosecond, Some("UTC".into()))
            }
            DataType::Binary => ArrowDataType::Binary,
            DataType::String => ArrowDataType::Utf8,
            DataType::Boolean => ArrowDataType::Boolean,
        }
    }
}

#[cfg(feature = "duckdb")]
impl DataType {
    pub fn from_duckdb_type(value: &String) -> Result<DataType> {
        match value.as_str() {
            "TINYINT" => Ok(DataType::Int8),
            "SMALLINT" => Ok(DataType::Int16),
            "INTEGER" => Ok(DataType::Int32),
            "BIGINT" => Ok(DataType::Int64),
            "UTINYINT" => Ok(DataType::UInt8),
            "USMALLINT" => Ok(DataType::UInt16),
            "UINTEGER" => Ok(DataType::UInt32),
            "UBIGINT" => Ok(DataType::UInt64),
            "FLOAT" => Ok(DataType::Float32),
            "DOUBLE" => Ok(DataType::Float64),
            "TIMESTAMP_NS" => Ok(DataType::TimestampNs),
            "BLOB" => Ok(DataType::Binary),
            "VARCHAR" => Ok(DataType::String),
            "BOOLEAN" => Ok(DataType::Boolean),
            _ => Err(anyhow!("Could not convert type")),
        }
    }

    pub fn to_sql(&self, value: &Value) -> Result<Box<dyn ToSql>> {
        match value {
            Value::String(s) => match self {
                DataType::Binary => {
                    let decoded = BASE64_STANDARD.decode(s)?;
                    Ok(Box::new(decoded))
                }
                _ => Ok(Box::new(s.clone())),
            },
            Value::Number(n) => match self {
                DataType::TimestampNs => {
                    let ts = n.as_i64().ok_or(anyhow!("Could not convert time to i64"))?;
                    Ok(Box::new(DateTime::from_timestamp_nanos(ts)))
                }
                _ => Ok(Box::new(format!("{}", n))),
            },
            Value::Null => Ok(Box::new(duckdb::types::Value::Null)),
            x => Ok(Box::new(format!("{}", x))),
        }
    }

    pub fn to_duckdb_type(&self) -> &'static str {
        match self {
            DataType::Int8 => &"TINYINT",
            DataType::Int16 => &"SMALLINT",
            DataType::Int32 => &"INTEGER",
            DataType::Int64 => &"BIGINT",
            DataType::UInt8 => &"UTINYINT",
            DataType::UInt16 => &"USMALLINT",
            DataType::UInt32 => &"UINTEGER",
            DataType::UInt64 => &"UBIGINT",
            DataType::Float32 => &"FLOAT",
            DataType::Float64 => &"DOUBLE",
            DataType::TimestampNs => &"TIMESTAMP_NS",
            DataType::Binary => &"BLOB",
            DataType::String => &"VARCHAR",
            DataType::Boolean => &"BOOLEAN",
        }
    }
}
