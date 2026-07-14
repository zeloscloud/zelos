use async_trait::async_trait;
use serde_json::{json, Value};

use crate::{ActionExecuteResult, ActionsError};

/// Trait for actions that can be executed
#[async_trait]
pub trait Action: Send + Sync {
    /// Execute the action with given parameters
    ///
    /// Returns:
    /// - Ok(ActionExecuteResult) for completed execution (with pass/fail status)
    /// - Err(error) for system-level execution failures
    async fn execute(&self, params: Value) -> Result<ActionExecuteResult, ActionsError>;

    /// Get the schema for this action
    /// Returns (json_schema, ui_schema, description)
    fn get_schema(
        &self,
        _current_values: Option<Value>,
    ) -> Result<(Value, Value, String), ActionsError> {
        // Default implementation provides a basic schema
        Ok((
            json!({
                "type": "object",
                "properties": {},
                "title": "Action",
                "description": "An executable action"
            }),
            json!({}),
            "An executable action".to_string(),
        ))
    }

    /// Get the schema for this action as raw JSON strings, preserving property order.
    ///
    /// Many form renderers display fields in the order the schema's `properties`
    /// are declared, so an action's schema must travel through the pipeline as an
    /// opaque string to avoid `serde_json::Map` (BTreeMap) reordering keys
    /// alphabetically on every parse/serialize. Implementors whose schema
    /// originates in an order-preserving representation should override this to
    /// return strings directly and skip the `Value` round-trip.
    ///
    /// Returns `(json_schema, ui_schema, description)` where the first two are
    /// JSON-encoded strings.
    fn get_schema_json(
        &self,
        current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        let (json_schema, ui_schema, description) = self.get_schema(current_values)?;
        let json_schema_str = serde_json::to_string(&json_schema).map_err(|e| {
            ActionsError::ExecutionError(format!("Failed to serialize JSON schema: {}", e))
        })?;
        let ui_schema_str = serde_json::to_string(&ui_schema).map_err(|e| {
            ActionsError::ExecutionError(format!("Failed to serialize UI schema: {}", e))
        })?;
        Ok((json_schema_str, ui_schema_str, description))
    }

    /// Get the action-defined default timeout in milliseconds, if any.
    /// Returns None to use the server default.
    fn default_timeout_ms(&self) -> Option<u32> {
        None
    }
}
