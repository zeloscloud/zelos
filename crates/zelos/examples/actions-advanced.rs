//! Application-driven cancellation of an in-flight action.
//!
//! This example serves a single action, `long_task`, built with [`ActionFn`]:
//! it sleeps for the requested number of seconds. A small RAII guard prints
//! `Long task cancelled` if the action's future is dropped before it finishes,
//! which happens when `serve` is cancelled mid-execution.
//!
//! Run with `just example rust actions-advanced`, then in another terminal:
//! `zelos actions execute rust-advanced/long_task --params '{"seconds":30}'`.
//! Press Ctrl-C in this provider while the invocation runs to observe
//! `Long task cancelled` as the in-flight execution is cancelled.

use std::{sync::Arc, time::Duration};

use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;
use zelos::actions::ActionsError;
use zelos::{ActionExecuteResult, ActionFn, ActionSchema, ActionsClient, ActionsRegistry};

/// Prints `Long task cancelled` when dropped, unless `disarm` was called first.
struct CancelGuard {
    done: bool,
}

impl CancelGuard {
    fn new() -> Self {
        Self { done: false }
    }

    fn disarm(&mut self) {
        self.done = true;
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        if !self.done {
            println!("Long task cancelled");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    println!("Connecting to Zelos agent at: {}", url);

    let long_task = ActionFn::new(
        ActionSchema::new("Long Task", "Sleep for the requested number of seconds")
            .integer("seconds", |f| f.title("Seconds").minimum(1.0).required()),
        |params: Value| async move {
            let seconds = params
                .get("seconds")
                .and_then(Value::as_u64)
                .filter(|seconds| *seconds > 0)
                .ok_or_else(|| {
                    ActionsError::ExecutionError("'seconds' must be a positive integer".to_string())
                })?;

            // The guard fires on drop if we're cancelled during the sleep.
            let mut guard = CancelGuard::new();
            tokio::time::sleep(Duration::from_secs(seconds)).await;
            guard.disarm();

            Ok(ActionExecuteResult::done(
                &json!({ "slept_seconds": seconds }),
            ))
        },
    );

    let registry = Arc::new(ActionsRegistry::new());
    registry.register("long_task".to_string(), Arc::new(long_task));

    let client = ActionsClient::new_with_url(url)?;
    let cancellation_token = CancellationToken::new();

    println!("Serving action as 'rust-advanced/long_task'");
    println!("Press Ctrl-C to stop.");

    tokio::select! {
        result = client.serve("rust-advanced".to_string(), registry, cancellation_token.clone()) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down...");
            cancellation_token.cancel();
        }
    }

    Ok(())
}
