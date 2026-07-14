//! Register and serve custom actions to a Zelos agent.
//!
//! This example builds an [`ActionsRegistry`] with two actions and serves them
//! to a Zelos agent over gRPC:
//! - `add` — adds two numbers and returns their sum (Done)
//! - `check_threshold` — compares a value against a threshold (Pass/Fail)
//!
//! Once running, the actions appear on the agent as `rust-example/add` and
//! `rust-example/check_threshold`.
//!
//! Run with `just example rust actions` (optionally pass a custom agent URL).

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;
use zelos::actions::ActionsError;
use zelos::{Action, ActionExecuteResult, ActionSchema, ActionsClient, ActionsRegistry};

/// Adds two numbers and returns their sum with a `Done` status.
struct AddNumbers;

#[async_trait]
impl Action for AddNumbers {
    async fn execute(&self, params: Value) -> Result<ActionExecuteResult, ActionsError> {
        let x = params
            .get("x")
            .and_then(Value::as_f64)
            .ok_or_else(|| ActionsError::ExecutionError("missing or invalid 'x'".to_string()))?;
        let y = params
            .get("y")
            .and_then(Value::as_f64)
            .ok_or_else(|| ActionsError::ExecutionError("missing or invalid 'y'".to_string()))?;

        Ok(ActionExecuteResult::done(&json!({ "sum": x + y })))
    }

    fn get_schema_json(
        &self,
        _current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        Ok(
            ActionSchema::new("Add Numbers", "Add two numbers and return their sum")
                .number("x", |f| f.title("X").required())
                .number("y", |f| f.title("Y").required())
                .to_schema_json(),
        )
    }
}

/// Returns `Pass` when `value <= threshold`, otherwise `Fail`.
struct CheckThreshold;

#[async_trait]
impl Action for CheckThreshold {
    async fn execute(&self, params: Value) -> Result<ActionExecuteResult, ActionsError> {
        let value = params.get("value").and_then(Value::as_f64).ok_or_else(|| {
            ActionsError::ExecutionError("missing or invalid 'value'".to_string())
        })?;
        let threshold = params
            .get("threshold")
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                ActionsError::ExecutionError("missing or invalid 'threshold'".to_string())
            })?;

        let result = json!({ "value": value, "threshold": threshold });
        if value <= threshold {
            Ok(ActionExecuteResult::passed(&result))
        } else {
            Ok(ActionExecuteResult::failed(&result))
        }
    }

    fn get_schema_json(
        &self,
        _current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        Ok(ActionSchema::new(
            "Check Threshold",
            "Check whether a value is within a threshold",
        )
        .number("value", |f| f.title("Value").required())
        .number("threshold", |f| f.title("Threshold").required())
        .to_schema_json())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configuration
    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    println!("Connecting to Zelos agent at: {}", url);

    // Register the actions under simple paths
    let registry = Arc::new(ActionsRegistry::new());
    registry.register("add".to_string(), Arc::new(AddNumbers));
    registry.register("check_threshold".to_string(), Arc::new(CheckThreshold));

    // Create the actions client and serve until Ctrl-C
    let client = ActionsClient::new_with_url(url)?;
    let cancellation_token = CancellationToken::new();

    println!("Serving actions as 'rust-example/add' and 'rust-example/check_threshold'");
    println!("Press Ctrl-C to stop.");

    tokio::select! {
        result = client.serve("rust-example".to_string(), registry, cancellation_token.clone()) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down...");
            cancellation_token.cancel();
        }
    }

    Ok(())
}
