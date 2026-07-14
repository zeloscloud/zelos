use std::{future::Future, pin::Pin};

use async_trait::async_trait;
use serde_json::Value;

use crate::{Action, ActionExecuteResult, ActionSchema, ActionsError};

type ActionFuture = Pin<Box<dyn Future<Output = Result<ActionExecuteResult, ActionsError>> + Send>>;

/// An [`Action`] backed by an async closure, so a simple action needs no struct
/// plus a hand-written `impl Action`.
///
/// The schema is carried by an [`ActionSchema`] so property order is preserved
/// through [`Action::get_schema_json`].
pub struct ActionFn {
    schema: ActionSchema,
    handler: Box<dyn Fn(Value) -> ActionFuture + Send + Sync>,
}

impl ActionFn {
    /// Build an action from a `schema` and an async `handler` closure.
    pub fn new<F, Fut>(schema: ActionSchema, handler: F) -> Self
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<ActionExecuteResult, ActionsError>> + Send + 'static,
    {
        Self {
            schema,
            handler: Box::new(move |params| Box::pin(handler(params))),
        }
    }
}

#[async_trait]
impl Action for ActionFn {
    async fn execute(&self, params: Value) -> Result<ActionExecuteResult, ActionsError> {
        (self.handler)(params).await
    }

    fn get_schema_json(
        &self,
        _current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        Ok(self.schema.to_schema_json())
    }
}
