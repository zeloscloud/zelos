use anyhow::{anyhow, Result};
use base64::prelude::*;
use std::fmt;

use crate::DataType;

/// An enum for wrapping primitive types as generics.
#[derive(Debug, Clone)]
pub enum Value {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    TimestampNs(i64),
    Binary(Vec<u8>),
    String(String),
    Boolean(bool),
}

// Manually implement PartialEq for Value so we can make float NaN == NaN
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int8(a), Value::Int8(b)) => a == b,
            (Value::Int16(a), Value::Int16(b)) => a == b,
            (Value::Int32(a), Value::Int32(b)) => a == b,
            (Value::Int64(a), Value::Int64(b)) => a == b,
            (Value::UInt8(a), Value::UInt8(b)) => a == b,
            (Value::UInt16(a), Value::UInt16(b)) => a == b,
            (Value::UInt32(a), Value::UInt32(b)) => a == b,
            (Value::UInt64(a), Value::UInt64(b)) => a == b,
            (Value::Float32(a), Value::Float32(b)) => a.to_bits() == b.to_bits(),
            (Value::Float64(a), Value::Float64(b)) => a.to_bits() == b.to_bits(),
            (Value::TimestampNs(a), Value::TimestampNs(b)) => a == b,
            (Value::Binary(a), Value::Binary(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            _ => false,
        }
    }
}

// Implement Eq for Value so we can iterate to build a hash map.
impl Eq for Value {}

// Implement Hash so we can use it as a key in a hash map.
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Int8(v) => {
                0_i8.hash(state);
                v.hash(state);
            }
            Value::Int16(v) => {
                1_i8.hash(state);
                v.hash(state);
            }
            Value::Int32(v) => {
                2_i8.hash(state);
                v.hash(state);
            }
            Value::Int64(v) => {
                3_i8.hash(state);
                v.hash(state);
            }
            Value::UInt8(v) => {
                4_i8.hash(state);
                v.hash(state);
            }
            Value::UInt16(v) => {
                5_i8.hash(state);
                v.hash(state);
            }
            Value::UInt32(v) => {
                6_i8.hash(state);
                v.hash(state);
            }
            Value::UInt64(v) => {
                7_i8.hash(state);
                v.hash(state);
            }
            Value::Float32(v) => {
                8_i8.hash(state);
                v.to_bits().hash(state);
            }
            Value::Float64(v) => {
                9_i8.hash(state);
                v.to_bits().hash(state);
            }
            Value::TimestampNs(v) => {
                10_i8.hash(state);
                v.hash(state);
            }
            Value::Binary(v) => {
                11_i8.hash(state);
                v.hash(state);
            }
            Value::String(v) => {
                12_i8.hash(state);
                v.hash(state);
            }
            Value::Boolean(v) => {
                13_i8.hash(state);
                v.hash(state);
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int8(v) => write!(f, "{}", v),
            Value::Int16(v) => write!(f, "{}", v),
            Value::Int32(v) => write!(f, "{}", v),
            Value::Int64(v) => write!(f, "{}", v),
            Value::UInt8(v) => write!(f, "{}", v),
            Value::UInt16(v) => write!(f, "{}", v),
            Value::UInt32(v) => write!(f, "{}", v),
            Value::UInt64(v) => write!(f, "{}", v),
            Value::Float32(v) => write!(f, "{}", v),
            Value::Float64(v) => write!(f, "{}", v),
            Value::TimestampNs(v) => write!(f, "{}", v),
            Value::Binary(items) => write!(f, "{}", BASE64_STANDARD.encode(items)),
            Value::String(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
        }
    }
}

impl Value {
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Int8(_) => DataType::Int8,
            Value::Int16(_) => DataType::Int16,
            Value::Int32(_) => DataType::Int32,
            Value::Int64(_) => DataType::Int64,
            Value::UInt8(_) => DataType::UInt8,
            Value::UInt16(_) => DataType::UInt16,
            Value::UInt32(_) => DataType::UInt32,
            Value::UInt64(_) => DataType::UInt64,
            Value::Float32(_) => DataType::Float32,
            Value::Float64(_) => DataType::Float64,
            Value::TimestampNs(_) => DataType::TimestampNs,
            Value::Binary(_) => DataType::Binary,
            Value::String(_) => DataType::String,
            Value::Boolean(_) => DataType::Boolean,
        }
    }

    pub fn as_i8(&self) -> Option<i8> {
        match self {
            Value::Int8(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            Value::Int16(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Int32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u8(&self) -> Option<u8> {
        match self {
            Value::UInt8(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u16(&self) -> Option<u16> {
        match self {
            Value::UInt16(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Value::UInt32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::UInt64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::Float32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_timestamp_ns(&self) -> Option<i64> {
        match self {
            Value::TimestampNs(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            Value::Binary(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<serde_json::Number> {
        match self {
            Value::Int8(v) => Some(serde_json::Number::from(*v)),
            Value::Int16(v) => Some(serde_json::Number::from(*v)),
            Value::Int32(v) => Some(serde_json::Number::from(*v)),
            Value::Int64(v) => Some(serde_json::Number::from(*v)),
            Value::UInt8(v) => Some(serde_json::Number::from(*v)),
            Value::UInt16(v) => Some(serde_json::Number::from(*v)),
            Value::UInt32(v) => Some(serde_json::Number::from(*v)),
            Value::UInt64(v) => Some(serde_json::Number::from(*v)),
            _ => None,
        }
    }

    pub fn from_number_as_type(value: &serde_json::Number, data_type: &DataType) -> Option<Value> {
        match data_type {
            DataType::Int8 => value
                .as_i64()
                .and_then(|v| v.try_into().ok())
                .map(Value::Int8),
            DataType::Int16 => value
                .as_i64()
                .and_then(|v| v.try_into().ok())
                .map(Value::Int16),
            DataType::Int32 => value
                .as_i64()
                .and_then(|v| v.try_into().ok())
                .map(Value::Int32),
            DataType::Int64 => value.as_i64().map(Value::Int64),
            DataType::UInt8 => value
                .as_u64()
                .and_then(|v| v.try_into().ok())
                .map(Value::UInt8),
            DataType::UInt16 => value
                .as_u64()
                .and_then(|v| v.try_into().ok())
                .map(Value::UInt16),
            DataType::UInt32 => value
                .as_u64()
                .and_then(|v| v.try_into().ok())
                .map(Value::UInt32),
            DataType::UInt64 => value.as_u64().map(Value::UInt64),
            _ => None,
        }
    }

    pub fn try_from_serde_json_as_type(
        value: serde_json::Value,
        data_type: &DataType,
    ) -> Result<Value> {
        match (data_type, value) {
            (DataType::Int8, serde_json::Value::Number(value)) => {
                let v = value
                    .as_i64()
                    .ok_or(anyhow!("Unable to read number as i64"))?
                    .try_into()?;
                Ok(Value::Int8(v))
            }
            (DataType::Int16, serde_json::Value::Number(value)) => {
                let v = value
                    .as_i64()
                    .ok_or(anyhow!("Unable to read number as i64"))?
                    .try_into()?;
                Ok(Value::Int16(v))
            }
            (DataType::Int32, serde_json::Value::Number(value)) => {
                let v = value
                    .as_i64()
                    .ok_or(anyhow!("Unable to read number as i64"))?
                    .try_into()?;
                Ok(Value::Int32(v))
            }
            (DataType::Int64, serde_json::Value::Number(value)) => {
                let v = value
                    .as_i64()
                    .ok_or(anyhow!("Unable to read number as i64"))?;
                Ok(Value::Int64(v))
            }
            (DataType::UInt8, serde_json::Value::Number(value)) => {
                let v = value
                    .as_u64()
                    .ok_or(anyhow!("Unable to read number as i64"))?
                    .try_into()?;
                Ok(Value::UInt8(v))
            }
            (DataType::UInt16, serde_json::Value::Number(value)) => {
                let v = value
                    .as_u64()
                    .ok_or(anyhow!("Unable to read number as i64"))?
                    .try_into()?;
                Ok(Value::UInt16(v))
            }
            (DataType::UInt32, serde_json::Value::Number(value)) => {
                let v = value
                    .as_u64()
                    .ok_or(anyhow!("Unable to read number as u32"))?
                    .try_into()?;
                Ok(Value::UInt32(v))
            }
            (DataType::UInt64, serde_json::Value::Number(value)) => {
                let v = value
                    .as_u64()
                    .ok_or(anyhow!("Unable to read number as u64"))?;
                Ok(Value::UInt64(v))
            }
            (DataType::Float32, serde_json::Value::Number(value)) => {
                let v = value
                    .as_f64()
                    .ok_or(anyhow!("Unable to read number as f64"))?;
                Ok(Value::Float32(v as f32))
            }
            (DataType::Float64, serde_json::Value::Number(value)) => {
                let v = value
                    .as_f64()
                    .ok_or(anyhow!("Unable to read number as f64"))?;
                Ok(Value::Float64(v))
            }
            (DataType::TimestampNs, serde_json::Value::Number(value)) => {
                let v = value
                    .as_i64()
                    .ok_or(anyhow!("Unable to read number as i64"))?;
                Ok(Value::TimestampNs(v))
            }
            (DataType::String, serde_json::Value::String(value)) => Ok(Value::String(value)),
            (DataType::Binary, serde_json::Value::String(value)) => {
                let decoded = BASE64_STANDARD.decode(value)?;
                Ok(Value::Binary(decoded))
            }
            (DataType::Boolean, serde_json::Value::Bool(value)) => Ok(Value::Boolean(value)),
            (to_type, from_value) => Err(anyhow!(
                "Unsupported data type conversion: {:?} to {:?}",
                from_value,
                to_type,
            )),
        }
    }

    pub fn try_to_serde_json(value: Value) -> Result<serde_json::Value> {
        match value {
            Value::Int8(v) => Ok(serde_json::Value::Number(v.into())),
            Value::Int16(v) => Ok(serde_json::Value::Number(v.into())),
            Value::Int32(v) => Ok(serde_json::Value::Number(v.into())),
            Value::Int64(v) => Ok(serde_json::Value::Number(v.into())),
            Value::UInt8(v) => Ok(serde_json::Value::Number(v.into())),
            Value::UInt16(v) => Ok(serde_json::Value::Number(v.into())),
            Value::UInt32(v) => Ok(serde_json::Value::Number(v.into())),
            Value::UInt64(v) => Ok(serde_json::Value::Number(v.into())),
            Value::Float32(v) => serde_json::Number::from_f64(v as f64)
                .map(serde_json::Value::Number)
                .ok_or(anyhow!("Unable to convert float32 to serde_json::Value")),
            Value::Float64(v) => serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .ok_or(anyhow!("Unable to convert float64 to serde_json::Value")),
            Value::TimestampNs(v) => Ok(serde_json::Value::Number(v.into())),
            Value::Binary(v) => Ok(serde_json::Value::String(BASE64_STANDARD.encode(v))),
            Value::String(v) => Ok(serde_json::Value::String(v)),
            Value::Boolean(v) => Ok(serde_json::Value::Bool(v)),
        }
    }
}
