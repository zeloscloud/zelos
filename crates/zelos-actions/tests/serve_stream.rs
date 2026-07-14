//! In-process integration test for `ActionsClient::serve`.
//!
//! Spins up a tonic server implementing only the test side of the `Actions`
//! bidirectional stream, connects a real client, and drives a scripted
//! conversation (handshake, list, execute, timeout, schema).

use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::{TcpListenerStream, UnboundedReceiverStream};
use tokio_stream::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status, Streaming};

use zelos_actions::{Action, ActionExecuteResult, ActionsClient, ActionsError, ActionsRegistry};
use zelos_proto::actions::{
    actions_response,
    actions_server::{Actions, ActionsServer},
    ActionsRequest, ActionsResponse, ExecuteRequest, ListRequest, SchemaRequest,
};

const SERVICE_NAME: &str = "test-service";

// --- Test-side actions registered on the client ---------------------------

struct OkAction;

#[async_trait]
impl Action for OkAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        Ok(ActionExecuteResult::passed(&json!({ "ok": true })))
    }
}

struct FailAction;

#[async_trait]
impl Action for FailAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        Err(ActionsError::ExecutionError("boom".to_string()))
    }
}

struct SlowAction;

#[async_trait]
impl Action for SlowAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok(ActionExecuteResult::done(&json!({})))
    }
}

/// Fails with a message containing quotes, backslashes, and newlines to prove
/// error payloads are always serialized as valid JSON.
struct HostileErrorAction;

const HOSTILE_MESSAGE: &str = "boom \"quoted\" \\path\\ and\nnewline";

#[async_trait]
impl Action for HostileErrorAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        Err(ActionsError::ExecutionError(HOSTILE_MESSAGE.to_string()))
    }
}

/// Sleeps briefly before passing; used to prove `timeout_ms == 0` is treated as
/// "no timeout" (a real 0ms timeout would elapse before this completes).
struct DelayOkAction;

#[async_trait]
impl Action for DelayOkAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(ActionExecuteResult::passed(&json!({ "ok": true })))
    }
}

// --- Test-side server: drives the scripted conversation -------------------

struct TestServer {
    report_tx: mpsc::UnboundedSender<ActionsRequest>,
}

fn list_request(seq: u64) -> ActionsResponse {
    ActionsResponse {
        sequence_number: seq,
        msg: Some(actions_response::Msg::List(ListRequest {})),
    }
}

fn execute_request(seq: u64, action: &str, timeout_ms: Option<u32>) -> ActionsResponse {
    execute_request_with_params(seq, action, "{}", timeout_ms)
}

fn execute_request_with_params(
    seq: u64,
    action: &str,
    params: &str,
    timeout_ms: Option<u32>,
) -> ActionsResponse {
    ActionsResponse {
        sequence_number: seq,
        msg: Some(actions_response::Msg::Execute(ExecuteRequest {
            action: action.to_string(),
            params: params.to_string(),
            timeout_ms,
        })),
    }
}

fn schema_request(seq: u64, action: &str) -> ActionsResponse {
    ActionsResponse {
        sequence_number: seq,
        msg: Some(actions_response::Msg::Schema(SchemaRequest {
            action: action.to_string(),
            current_values: String::new(),
        })),
    }
}

/// Read client messages until one echoes `seq`, ignoring periodic pushes (seq 0).
async fn read_until(stream: &mut Streaming<ActionsRequest>, seq: u64) -> Option<ActionsRequest> {
    while let Ok(Some(req)) = stream.message().await {
        if req.sequence_number == seq {
            return Some(req);
        }
    }
    None
}

#[async_trait]
impl Actions for TestServer {
    async fn execute(
        &self,
        _request: Request<ExecuteRequest>,
    ) -> Result<Response<zelos_proto::actions::ExecuteResponse>, Status> {
        Err(Status::unimplemented("unary execute not used in this test"))
    }

    async fn list(
        &self,
        _request: Request<ListRequest>,
    ) -> Result<Response<zelos_proto::actions::ListResponse>, Status> {
        Err(Status::unimplemented("unary list not used in this test"))
    }

    async fn schema(
        &self,
        _request: Request<SchemaRequest>,
    ) -> Result<Response<zelos_proto::actions::SchemaResponse>, Status> {
        Err(Status::unimplemented("unary schema not used in this test"))
    }

    type ActionsStream = Pin<Box<dyn Stream<Item = Result<ActionsResponse, Status>> + Send>>;

    async fn actions(
        &self,
        request: Request<Streaming<ActionsRequest>>,
    ) -> Result<Response<Self::ActionsStream>, Status> {
        let mut inbound = request.into_inner();
        let report_tx = self.report_tx.clone();
        let (tx, rx) = mpsc::unbounded_channel::<ActionsResponse>();

        tokio::spawn(async move {
            // 1. Handshake (first client message).
            if let Ok(Some(handshake)) = inbound.message().await {
                let _ = report_tx.send(handshake);
            } else {
                return;
            }

            // 2. List.
            let _ = tx.send(list_request(1));
            if let Some(resp) = read_until(&mut inbound, 1).await {
                let _ = report_tx.send(resp);
            }

            // 3. Execute a passing action.
            let _ = tx.send(execute_request(2, "test/ok", None));
            if let Some(resp) = read_until(&mut inbound, 2).await {
                let _ = report_tx.send(resp);
            }

            // 4. Execute a failing action.
            let _ = tx.send(execute_request(3, "test/fail", None));
            if let Some(resp) = read_until(&mut inbound, 3).await {
                let _ = report_tx.send(resp);
            }

            // 5. Execute a slow action with a short timeout.
            let _ = tx.send(execute_request(4, "test/slow", Some(50)));
            if let Some(resp) = read_until(&mut inbound, 4).await {
                let _ = report_tx.send(resp);
            }

            // 6. Schema.
            let _ = tx.send(schema_request(5, "test/ok"));
            if let Some(resp) = read_until(&mut inbound, 5).await {
                let _ = report_tx.send(resp);
            }

            // 7. Execute with malformed params: expect a correlated error response.
            let _ = tx.send(execute_request_with_params(6, "test/ok", "{not json", None));
            if let Some(resp) = read_until(&mut inbound, 6).await {
                let _ = report_tx.send(resp);
            }

            // 8. Execute an action that fails with a hostile error message.
            let _ = tx.send(execute_request(7, "test/hostile", None));
            if let Some(resp) = read_until(&mut inbound, 7).await {
                let _ = report_tx.send(resp);
            }

            // 9. Execute a delayed action with timeout_ms == 0 (treated as absent).
            let _ = tx.send(execute_request(8, "test/delay", Some(0)));
            if let Some(resp) = read_until(&mut inbound, 8).await {
                let _ = report_tx.send(resp);
            }

            // Keep the response stream open so the client shuts down via its
            // cancellation token rather than hitting the reconnect path.
            tokio::time::sleep(Duration::from_secs(3600)).await;
            drop(tx);
        });

        let stream = UnboundedReceiverStream::new(rx).map(Ok::<ActionsResponse, Status>);
        Ok(Response::new(Box::pin(stream)))
    }
}

async fn recv(rx: &mut mpsc::UnboundedReceiver<ActionsRequest>) -> ActionsRequest {
    tokio::time::timeout(Duration::from_secs(10), rx.recv())
        .await
        .expect("timed out waiting for report")
        .expect("report channel closed")
}

fn execute_response(req: &ActionsRequest) -> &zelos_proto::actions::ExecuteResponse {
    match req.msg.as_ref() {
        Some(zelos_proto::actions::actions_request::Msg::Execute(e)) => e,
        other => panic!("expected execute response, got {:?}", other),
    }
}

#[tokio::test]
async fn serve_stream_round_trip() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let (report_tx, mut report_rx) = mpsc::unbounded_channel::<ActionsRequest>();
    let server = TestServer { report_tx };

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ActionsServer::new(server))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    let client = Arc::new(ActionsClient::new_with_url(format!("http://{}", addr)).unwrap());

    let registry = Arc::new(ActionsRegistry::new());
    registry.register("test/ok".to_string(), Arc::new(OkAction));
    registry.register("test/fail".to_string(), Arc::new(FailAction));
    registry.register("test/slow".to_string(), Arc::new(SlowAction));
    registry.register("test/hostile".to_string(), Arc::new(HostileErrorAction));
    registry.register("test/delay".to_string(), Arc::new(DelayOkAction));

    let token = CancellationToken::new();
    let serve_handle = {
        let client = client.clone();
        let registry = registry.clone();
        let token = token.clone();
        tokio::spawn(async move {
            client
                .serve(SERVICE_NAME.to_string(), registry, token)
                .await
        })
    };

    // 1. Handshake.
    let handshake = recv(&mut report_rx).await;
    assert_eq!(handshake.sequence_number, 0);
    assert_eq!(handshake.name, SERVICE_NAME);
    assert!(handshake.msg.is_none());

    // 2. List response echoes seq 1 and lists all registered actions.
    let list = recv(&mut report_rx).await;
    assert_eq!(list.sequence_number, 1);
    match list.msg {
        Some(zelos_proto::actions::actions_request::Msg::List(l)) => {
            assert!(l.actions.contains(&"test/ok".to_string()));
            assert!(l.actions.contains(&"test/fail".to_string()));
            assert!(l.actions.contains(&"test/slow".to_string()));
        }
        other => panic!("expected list response, got {:?}", other),
    }

    // 3. Passing execute.
    let ok = recv(&mut report_rx).await;
    assert_eq!(ok.sequence_number, 2);
    let ok_exec = execute_response(&ok);
    assert_eq!(ok_exec.action, "test/ok");
    assert_eq!(
        ok_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Pass as i32)
    );
    let ok_value: Value = serde_json::from_str(&ok_exec.result).unwrap();
    assert_eq!(ok_value, json!({ "ok": true }));

    // 4. Failing execute -> Error status with {"error": ...}.
    let fail = recv(&mut report_rx).await;
    assert_eq!(fail.sequence_number, 3);
    let fail_exec = execute_response(&fail);
    assert_eq!(fail_exec.action, "test/fail");
    assert_eq!(
        fail_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Error as i32)
    );
    let fail_value: Value = serde_json::from_str(&fail_exec.result).unwrap();
    assert!(
        fail_value.get("error").is_some(),
        "error key present: {}",
        fail_exec.result
    );

    // 5. Timeout -> Error status.
    let timeout = recv(&mut report_rx).await;
    assert_eq!(timeout.sequence_number, 4);
    let timeout_exec = execute_response(&timeout);
    assert_eq!(timeout_exec.action, "test/slow");
    assert_eq!(
        timeout_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Error as i32)
    );
    assert!(
        timeout_exec.result.contains("timed out"),
        "result: {}",
        timeout_exec.result
    );

    // 6. Schema.
    let schema = recv(&mut report_rx).await;
    assert_eq!(schema.sequence_number, 5);
    match schema.msg {
        Some(zelos_proto::actions::actions_request::Msg::Schema(s)) => {
            assert_eq!(s.action, "test/ok");
            let parsed: Value = serde_json::from_str(&s.action_schema).unwrap();
            assert_eq!(parsed["type"], "object");
        }
        other => panic!("expected schema response, got {:?}", other),
    }

    // 7. Malformed params -> correlated error response preserving the sequence
    // number, with a valid-JSON error payload.
    let bad_params = recv(&mut report_rx).await;
    assert_eq!(bad_params.sequence_number, 6);
    let bad_exec = execute_response(&bad_params);
    assert_eq!(bad_exec.action, "test/ok");
    assert_eq!(
        bad_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Error as i32)
    );
    let bad_value: Value = serde_json::from_str(&bad_exec.result).unwrap();
    assert!(
        bad_value["error"]
            .as_str()
            .unwrap()
            .contains("Failed to parse params"),
        "result: {}",
        bad_exec.result
    );

    // 8. Hostile error message round-trips as valid JSON.
    let hostile = recv(&mut report_rx).await;
    assert_eq!(hostile.sequence_number, 7);
    let hostile_exec = execute_response(&hostile);
    assert_eq!(
        hostile_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Error as i32)
    );
    let hostile_value: Value = serde_json::from_str(&hostile_exec.result).unwrap();
    assert_eq!(
        hostile_value["error"].as_str().unwrap(),
        "Execution error: boom \"quoted\" \\path\\ and\nnewline"
    );

    // 9. timeout_ms == 0 falls back to no timeout, so the delayed action passes.
    let delay = recv(&mut report_rx).await;
    assert_eq!(delay.sequence_number, 8);
    let delay_exec = execute_response(&delay);
    assert_eq!(delay_exec.action, "test/delay");
    assert_eq!(
        delay_exec.status,
        Some(zelos_proto::actions::ExecuteStatus::Pass as i32),
        "timeout_ms=0 should be treated as absent; result: {}",
        delay_exec.result
    );

    token.cancel();
    serve_handle.await.unwrap().unwrap();
}

// --- In-flight action cancellation on stream teardown ---------------------

/// Signals when its future starts and again when the future is dropped, so a
/// test can observe that an in-flight action is aborted when serve is cancelled.
struct DropSignalAction {
    started_tx: mpsc::UnboundedSender<()>,
    dropped_tx: mpsc::UnboundedSender<()>,
}

#[async_trait]
impl Action for DropSignalAction {
    async fn execute(&self, _params: Value) -> Result<ActionExecuteResult, ActionsError> {
        struct DropGuard(mpsc::UnboundedSender<()>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                let _ = self.0.send(());
            }
        }
        let _guard = DropGuard(self.dropped_tx.clone());
        let _ = self.started_tx.send(());
        // Long-lived work; the guard fires if the future is dropped mid-await.
        tokio::time::sleep(Duration::from_secs(30)).await;
        Ok(ActionExecuteResult::done(&json!({})))
    }
}

/// Minimal server that, after the handshake, asks the client to execute a single
/// long-running action and then holds the stream open.
struct CancelServer;

#[async_trait]
impl Actions for CancelServer {
    async fn execute(
        &self,
        _request: Request<ExecuteRequest>,
    ) -> Result<Response<zelos_proto::actions::ExecuteResponse>, Status> {
        Err(Status::unimplemented("unary execute not used in this test"))
    }

    async fn list(
        &self,
        _request: Request<ListRequest>,
    ) -> Result<Response<zelos_proto::actions::ListResponse>, Status> {
        Err(Status::unimplemented("unary list not used in this test"))
    }

    async fn schema(
        &self,
        _request: Request<SchemaRequest>,
    ) -> Result<Response<zelos_proto::actions::SchemaResponse>, Status> {
        Err(Status::unimplemented("unary schema not used in this test"))
    }

    type ActionsStream = Pin<Box<dyn Stream<Item = Result<ActionsResponse, Status>> + Send>>;

    async fn actions(
        &self,
        request: Request<Streaming<ActionsRequest>>,
    ) -> Result<Response<Self::ActionsStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::unbounded_channel::<ActionsResponse>();

        tokio::spawn(async move {
            // Handshake, then request a long-running action.
            if inbound.message().await.is_err() {
                return;
            }
            let _ = tx.send(execute_request(1, "test/blocking", None));
            // Keep the stream open by draining inbound until the client hangs up.
            while let Ok(Some(_)) = inbound.message().await {}
        });

        let stream = UnboundedReceiverStream::new(rx).map(Ok::<ActionsResponse, Status>);
        Ok(Response::new(Box::pin(stream)))
    }
}

#[tokio::test]
async fn serve_cancel_aborts_in_flight_action() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ActionsServer::new(CancelServer))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    let client = Arc::new(ActionsClient::new_with_url(format!("http://{}", addr)).unwrap());

    let (started_tx, mut started_rx) = mpsc::unbounded_channel::<()>();
    let (dropped_tx, mut dropped_rx) = mpsc::unbounded_channel::<()>();

    let registry = Arc::new(ActionsRegistry::new());
    registry.register(
        "test/blocking".to_string(),
        Arc::new(DropSignalAction {
            started_tx,
            dropped_tx,
        }),
    );

    let token = CancellationToken::new();
    let serve_handle = {
        let client = client.clone();
        let registry = registry.clone();
        let token = token.clone();
        tokio::spawn(async move { client.serve(SERVICE_NAME.to_string(), registry, token).await })
    };

    // Wait until the action is actually running.
    tokio::time::timeout(Duration::from_secs(10), started_rx.recv())
        .await
        .expect("action did not start")
        .expect("started channel closed");

    // Cancelling serve should tear down the stream and abort the in-flight action.
    token.cancel();
    serve_handle.await.unwrap().unwrap();

    tokio::time::timeout(Duration::from_secs(10), dropped_rx.recv())
        .await
        .expect("in-flight action was not aborted")
        .expect("dropped channel closed");
}
