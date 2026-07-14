use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::Value;

use crate::{Action, ActionExecuteResult, ActionsError};

/// Registry for managing locally registered actions
#[derive(Clone)]
pub struct ActionsRegistry {
    // Local actions registered directly with this registry, keyed by full action path
    actions: Arc<RwLock<HashMap<String, Arc<dyn Action>>>>,
}

impl ActionsRegistry {
    /// Create a new empty actions registry
    pub fn new() -> Self {
        Self {
            actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a local action with its local path
    ///
    /// action_path: The full local action path, which can include namespaces like "namespace/class/action"
    pub fn register(&self, action_path: String, action: Arc<dyn Action>) {
        let mut actions = self.actions.write().unwrap();
        actions.insert(action_path, action);
    }

    /// Get a reference to a registered action by its full path
    pub fn get(&self, action_path: &str) -> Result<Arc<dyn Action>, ActionsError> {
        let actions = self
            .actions
            .read()
            .map_err(|e| ActionsError::ExecutionError(e.to_string()))?;

        actions
            .get(action_path)
            .cloned()
            .ok_or_else(|| ActionsError::NotFound(action_path.to_string()))
    }

    /// List all registered local actions as a vector of action paths.
    ///
    /// Sorted by action name so the customer-visible ordering is deterministic.
    pub fn list(&self) -> Result<Vec<String>, ActionsError> {
        let actions = self
            .actions
            .read()
            .map_err(|e| ActionsError::ExecutionError(e.to_string()))?;

        let mut list: Vec<String> = actions.keys().cloned().collect();
        list.sort();
        Ok(list)
    }

    /// Execute a local action with given parameters
    pub async fn execute(
        &self,
        action_path: &str,
        params: Value,
    ) -> Result<ActionExecuteResult, ActionsError> {
        self.get(action_path)?.execute(params).await
    }

    /// Get schema for a local action
    pub fn get_schema(
        &self,
        action_path: &str,
        current_values: Option<Value>,
    ) -> Result<(Value, Value, String), ActionsError> {
        let action = self.get(action_path)?;
        action.get_schema(current_values)
    }

    /// Get schema for a local action as raw JSON strings.
    ///
    /// Preserves property order end-to-end: implementors whose schema is already
    /// an order-preserving string can short-circuit the `serde_json::Value`
    /// conversion that would otherwise re-sort object keys alphabetically via
    /// `BTreeMap`.
    pub fn get_schema_json(
        &self,
        action_path: &str,
        current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        let action = self.get(action_path)?;
        action.get_schema_json(current_values)
    }

    /// Check if an action is registered locally
    pub fn contains(&self, action_path: &str) -> bool {
        self.actions
            .read()
            .map(|actions| actions.contains_key(action_path))
            .unwrap_or(false)
    }
}

impl Default for ActionsRegistry {
    fn default() -> Self {
        Self::new()
    }
}
