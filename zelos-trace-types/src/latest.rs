use serde::{Deserialize, Serialize};
#[cfg(feature = "ts-rs")]
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
pub struct SignalValue {
    pub full_name: String,
    pub signal: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(TS))]
pub struct LatestSignalData {
    pub message: String,
    pub timestamp: i64,
    pub values: Vec<SignalValue>,
}
