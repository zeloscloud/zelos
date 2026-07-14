use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::{
    Action, ActionExecuteResult, ActionFn, ActionSchema, ActionsError, ActionsRegistry,
    ExecuteStatus,
};

/// A minimal action that echoes a fixed result with PASS status.
struct EchoAction;

#[async_trait]
impl Action for EchoAction {
    async fn execute(&self, params: Value) -> Result<ActionExecuteResult, ActionsError> {
        Ok(ActionExecuteResult::passed(&json!({ "echo": params })))
    }
}

/// An action whose schema has intentionally non-alphabetical field order. It
/// overrides `get_schema_json` so the raw string flows through unchanged.
struct OrderedSchemaAction;

const ORDERED_SCHEMA_JSON: &str = r#"{"title":"add","type":"object","properties":{"x":{"type":"number"},"y":{"type":"number"},"a_later":{"type":"number"}}}"#;

#[async_trait]
impl Action for OrderedSchemaAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        Ok(ActionExecuteResult::done(&json!({})))
    }

    fn get_schema_json(
        &self,
        _current_values: Option<Value>,
    ) -> Result<(String, String, String), ActionsError> {
        Ok((
            ORDERED_SCHEMA_JSON.to_string(),
            "{}".to_string(),
            "ordered".to_string(),
        ))
    }
}

#[tokio::test]
async fn registry_round_trip() {
    let registry = ActionsRegistry::new();
    assert!(!registry.contains("test/echo"));

    registry.register("test/echo".to_string(), Arc::new(EchoAction));

    assert!(registry.contains("test/echo"));
    assert!(registry.get("test/echo").is_ok());

    let list = registry.list().unwrap();
    assert_eq!(list, vec!["test/echo".to_string()]);

    let result = registry
        .execute("test/echo", json!({ "a": 1 }))
        .await
        .unwrap();
    assert_eq!(result.status, ExecuteStatus::Pass);
    assert_eq!(result.value, json!({ "echo": { "a": 1 } }));
}

#[test]
fn registry_list_is_sorted_deterministically() {
    let registry = ActionsRegistry::new();
    // Register in deliberately non-sorted order.
    registry.register("ns/zebra".to_string(), Arc::new(EchoAction));
    registry.register("ns/apple".to_string(), Arc::new(EchoAction));
    registry.register("ns/mango".to_string(), Arc::new(EchoAction));

    let list = registry.list().unwrap();
    assert_eq!(
        list,
        vec![
            "ns/apple".to_string(),
            "ns/mango".to_string(),
            "ns/zebra".to_string(),
        ]
    );

    // Repeated calls yield identical ordering.
    assert_eq!(registry.list().unwrap(), list);
}

#[test]
fn registry_get_missing_is_not_found() {
    let registry = ActionsRegistry::new();
    match registry.get("missing/action") {
        Err(ActionsError::NotFound(path)) => assert_eq!(path, "missing/action"),
        Err(other) => panic!("expected NotFound, got {}", other),
        Ok(_) => panic!("expected NotFound error, got an action"),
    }
}

#[test]
fn result_into_execute_response_maps_status_and_json() {
    let cases = [
        (
            ActionExecuteResult::passed(&json!({ "k": 1 })),
            ExecuteStatus::Pass,
        ),
        (
            ActionExecuteResult::failed(&json!({ "k": 2 })),
            ExecuteStatus::Fail,
        ),
        (
            ActionExecuteResult::error(&json!({ "k": 3 })),
            ExecuteStatus::Error,
        ),
        (
            ActionExecuteResult::done(&json!({ "k": 4 })),
            ExecuteStatus::Done,
        ),
    ];

    for (result, expected_status) in cases {
        let expected_json = serde_json::to_string(&result.value).unwrap();
        let response = result.into_execute_response("ns/action");
        assert_eq!(response.action, "ns/action");
        assert_eq!(response.status, Some(expected_status as i32));
        assert_eq!(response.result, expected_json);
    }
}

#[test]
fn get_schema_json_override_preserves_property_order() {
    let action = OrderedSchemaAction;
    let (schema_json, ui_json, description) = action.get_schema_json(None).unwrap();

    // Overridden string flows through byte-for-byte.
    assert_eq!(schema_json, ORDERED_SCHEMA_JSON);
    assert_eq!(ui_json, "{}");
    assert_eq!(description, "ordered");

    // Property order is preserved (would be reordered by a serde_json::Value round-trip).
    let props = schema_json
        .split_once(r#""properties":{"#)
        .expect("properties key present")
        .1;
    let x = props.find(r#""x""#).unwrap();
    let y = props.find(r#""y""#).unwrap();
    let a_later = props.find(r#""a_later""#).unwrap();
    assert!(
        x < y && y < a_later,
        "x, y, a_later appear in declared order"
    );
}

#[tokio::test]
async fn action_fn_execute_invokes_closure() {
    let action = ActionFn::new(
        ActionSchema::new("Echo", "echoes params"),
        |params| async move { Ok(ActionExecuteResult::passed(&json!({ "echo": params }))) },
    );

    let result = action.execute(json!({ "a": 1 })).await.unwrap();
    assert_eq!(result.status, ExecuteStatus::Pass);
    assert_eq!(result.value, json!({ "echo": { "a": 1 } }));
}

#[test]
fn action_fn_get_schema_json_preserves_property_order() {
    // Fields declared "b" then "a": a Value round-trip would reorder them.
    let action = ActionFn::new(
        ActionSchema::new("Ordered", "b before a")
            .number("b", |f| f.required())
            .number("a", |f| f),
        |_params| async move { Ok(ActionExecuteResult::done(&json!({}))) },
    );

    let (schema_json, ui_json, description) = action.get_schema_json(None).unwrap();
    let b = schema_json.find(r#""b""#).unwrap();
    let a = schema_json.find(r#""a""#).unwrap();
    assert!(b < a, "b appears before a: {}", schema_json);
    assert_eq!(ui_json, "{}");
    assert_eq!(description, "b before a");
}

#[test]
fn default_get_schema_json_serializes_value_schema() {
    // EchoAction uses the default get_schema / get_schema_json implementations.
    let action = EchoAction;
    let (schema_json, ui_json, description) = action.get_schema_json(None).unwrap();

    let parsed: Value = serde_json::from_str(&schema_json).unwrap();
    assert_eq!(parsed["type"], "object");
    assert_eq!(parsed["title"], "Action");

    let ui: Value = serde_json::from_str(&ui_json).unwrap();
    assert_eq!(ui, json!({}));
    assert_eq!(description, "An executable action");

    assert_eq!(action.default_timeout_ms(), None);
}
