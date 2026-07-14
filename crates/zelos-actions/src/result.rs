use serde_json::Value;
use zelos_proto::actions::ExecuteResponse;

use crate::ExecuteStatus;

#[derive(Debug)]
pub struct ActionExecuteResult {
    pub value: Value,
    pub status: ExecuteStatus,
}

impl ActionExecuteResult {
    pub fn passed(value: &Value) -> Self {
        ActionExecuteResult {
            value: value.clone(),
            status: ExecuteStatus::Pass,
        }
    }
    pub fn failed(value: &Value) -> Self {
        ActionExecuteResult {
            value: value.clone(),
            status: ExecuteStatus::Fail,
        }
    }
    pub fn error(value: &Value) -> Self {
        ActionExecuteResult {
            value: value.clone(),
            status: ExecuteStatus::Error,
        }
    }
    pub fn done(value: &Value) -> Self {
        ActionExecuteResult {
            value: value.clone(),
            status: ExecuteStatus::Done,
        }
    }
}

impl ActionExecuteResult {
    pub(crate) fn into_execute_response(self, action: &str) -> ExecuteResponse {
        ExecuteResponse {
            action: action.to_string(),
            result: serde_json::to_string(&self.value)
                .unwrap_or_else(|_| "Failed to serialize result".to_string()),
            status: Some(self.status.into()),
        }
    }
}
