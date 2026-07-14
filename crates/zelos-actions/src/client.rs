use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};
use tokio_util::sync::CancellationToken;
use zelos_proto::actions::{
    actions_client::ActionsClient as GrpcActionsClient, ActionsRequest, ActionsResponse,
    ExecuteRequest, ExecuteResponse, ExecuteStatus, ListRequest, ListResponse, SchemaRequest,
    SchemaResponse, StatusResponse,
};
use zelos_proto::channel;

use crate::{error::ActionsError, ActionsRegistry};

/// Build a JSON error payload string (`{"error": <message>}`) via serde_json so
/// that messages containing quotes, backslashes, or newlines stay valid JSON.
fn error_json(message: impl Into<String>) -> String {
    serde_json::json!({ "error": message.into() }).to_string()
}

/// Client wrapper that provides convenient methods for calling the Actions service
pub struct ActionsClient {
    /// The gRPC channel
    channel: tonic::transport::Channel,
}

impl ActionsClient {
    /// Create a new ActionsClient using a shared channel
    pub fn new(channel: tonic::transport::Channel) -> Self {
        Self { channel }
    }

    /// Create a new ActionsClient with its own channel
    pub fn new_with_url(url: String) -> Result<Self, ActionsError> {
        let channel = channel::create_channel(url).map_err(|e| {
            ActionsError::ExecutionError(format!("Failed to create channel: {}", e))
        })?;
        Ok(Self::new(channel))
    }

    /// Helper to get a gRPC client
    fn get_client(&self) -> GrpcActionsClient<tonic::transport::Channel> {
        GrpcActionsClient::new(self.channel.clone())
            .max_decoding_message_size(zelos_proto::MAX_GRPC_MESSAGE_SIZE)
            .max_encoding_message_size(zelos_proto::MAX_GRPC_MESSAGE_SIZE)
    }

    pub async fn list(&self) -> Result<ListResponse, ActionsError> {
        let mut client = self.get_client();
        let request = ListRequest {};
        let response = client
            .list(request)
            .await
            .map_err(|e| ActionsError::ExecutionError(format!("gRPC error: {}", e)))?;

        Ok(response.into_inner())
    }

    pub async fn schema(
        &self,
        action: String,
        current_values: Option<Value>,
    ) -> Result<SchemaResponse, ActionsError> {
        let mut client = self.get_client();
        let request = SchemaRequest {
            action,
            current_values: current_values
                .map(|v| serde_json::to_string(&v).unwrap_or_default())
                .unwrap_or_default(),
        };
        let response = client
            .schema(request)
            .await
            .map_err(|e| ActionsError::ExecutionError(format!("gRPC error: {}", e)))?;

        Ok(response.into_inner())
    }

    pub async fn execute(
        &self,
        action: String,
        params: Value,
        timeout_ms: Option<u32>,
    ) -> Result<(Value, ExecuteStatus), ActionsError> {
        let mut client = self.get_client();
        let params_json = serde_json::to_string(&params).map_err(|e| {
            ActionsError::ExecutionError(format!("Failed to serialize params: {}", e))
        })?;

        let request = ExecuteRequest {
            action,
            params: params_json,
            timeout_ms,
        };

        let response = client
            .execute(request)
            .await
            .map_err(|e| ActionsError::ExecutionError(format!("gRPC error: {}", e)))?;

        let inner = response.into_inner();
        let result: Value = serde_json::from_str(&inner.result)
            .map_err(|e| ActionsError::ExecutionError(format!("Failed to parse result: {}", e)))?;
        let status = inner
            .status
            .and_then(|s| ExecuteStatus::try_from(s).ok())
            .unwrap_or(ExecuteStatus::Done);

        Ok((result, status))
    }

    /// Connect to the server and serve actions via bidirectional stream.
    /// Reconnects automatically if the connection is lost.
    ///
    /// This method establishes a bidirectional stream with the server and serves
    /// actions from the provided [`ActionsRegistry`]. It handles:
    /// - ListRequest: responds with available actions
    /// - ExecuteRequest: executes actions using the provided registry
    /// - StatusRequest: responds with service status
    /// - SchemaRequest: responds with action schemas
    pub async fn serve(
        &self,
        service_name: String,
        actions_registry: Arc<ActionsRegistry>,
        cancellation_token: CancellationToken,
    ) -> Result<(), ActionsError> {
        // Track when this serving lifecycle began so status responses can report
        // actual elapsed uptime rather than wall-clock time.
        let start = Instant::now();
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    tracing::debug!("Cancellation requested for actions client");
                    return Ok(());
                }
                result = Self::handle_actions_stream(
                    self.channel.clone(),
                    service_name.clone(),
                    actions_registry.clone(),
                    start,
                ) => {
                    match result {
                        Ok(()) => {
                            tracing::debug!("Actions stream ended normally");
                        }
                        Err(e) => {
                            tracing::warn!("Actions stream connection failed: {}", e);
                            // Back off before reconnecting, but exit promptly if
                            // cancelled during the wait.
                            tokio::select! {
                                _ = cancellation_token.cancelled() => {
                                    tracing::debug!("Cancellation requested during reconnect backoff");
                                    return Ok(());
                                }
                                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                            }
                        }
                    }
                }
            }
        }
    }

    /// Establish and handle a single bidirectional actions stream
    async fn handle_actions_stream(
        channel: tonic::transport::Channel,
        service_name: String,
        actions_registry: Arc<ActionsRegistry>,
        start: Instant,
    ) -> Result<(), ActionsError> {
        // Per-stream cancellation: cancelled when this function returns (stream
        // ended/errored) or is dropped (serve was cancelled), so in-flight
        // spawned action tasks are aborted rather than leaked.
        let stream_token = CancellationToken::new();
        let _stream_guard = stream_token.clone().drop_guard();

        let mut client = GrpcActionsClient::new(channel)
            .max_decoding_message_size(zelos_proto::MAX_GRPC_MESSAGE_SIZE)
            .max_encoding_message_size(zelos_proto::MAX_GRPC_MESSAGE_SIZE);

        tracing::debug!(
            "Starting bidirectional actions stream for service: {}",
            service_name
        );

        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let request_stream = UnboundedReceiverStream::new(request_rx);

        // Send initial NO-OP handshake message
        let handshake_message = ActionsRequest {
            sequence_number: 0,
            name: service_name.clone(),
            msg: None,
        };

        request_tx.send(handshake_message).map_err(|_| {
            ActionsError::ExecutionError("Failed to send initial handshake".to_string())
        })?;

        // Start the bidirectional stream
        let response_stream = client
            .actions(request_stream)
            .await
            .map_err(|e| {
                ActionsError::ExecutionError(format!("Failed to start actions stream: {}", e))
            })?
            .into_inner();

        let mut response_stream = Box::pin(response_stream);

        let mut list_interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                msg = response_stream.next() => {
                    match msg {
                        Some(Ok(response)) => {
                            if let Err(e) = Self::handle_server_message(
                                response,
                                &request_tx,
                                &service_name,
                                &actions_registry,
                                start,
                                &stream_token,
                            )
                            .await
                            {
                                tracing::error!("Error handling server message: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            tracing::error!("Stream error: {}", e);
                            return Err(ActionsError::ExecutionError(format!("Stream error: {}", e)));
                        }
                        None => {
                            // Stream ended - treat as connection failure to trigger retry
                            return Err(ActionsError::ExecutionError(
                                "Connection closed by server".to_string(),
                            ));
                        }
                    }
                }
                _ = list_interval.tick() => {
                    // Unconditionally push current action list (covers dynamically registered actions)
                    if let Err(e) = Self::handle_list_request(0, &request_tx, &actions_registry).await {
                        tracing::warn!("Failed to push action list: {}", e);
                    }
                }
            }
        }
    }

    async fn handle_server_message(
        response: ActionsResponse,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
        service_name: &str,
        actions_registry: &Arc<ActionsRegistry>,
        start: Instant,
        stream_token: &CancellationToken,
    ) -> Result<(), ActionsError> {
        let sequence_number = response.sequence_number;

        tracing::debug!(
            "Received message from server: sequence={}, msg_type={:?}",
            sequence_number,
            response.msg.as_ref().map(|m| match m {
                zelos_proto::actions::actions_response::Msg::List(_) => "List",
                zelos_proto::actions::actions_response::Msg::Execute(_) => "Execute",
                zelos_proto::actions::actions_response::Msg::Status(_) => "Status",
                zelos_proto::actions::actions_response::Msg::Schema(_) => "Schema",
            })
        );

        match response.msg {
            Some(zelos_proto::actions::actions_response::Msg::List(_list_request)) => {
                tracing::debug!("Received ListRequest from server, responding with actions");
                Self::handle_list_request(sequence_number, request_tx, actions_registry).await
            }
            Some(zelos_proto::actions::actions_response::Msg::Execute(execute_request)) => {
                Self::handle_execute_request(
                    sequence_number,
                    execute_request,
                    request_tx,
                    actions_registry,
                    stream_token,
                )
                .await
            }
            Some(zelos_proto::actions::actions_response::Msg::Status(_status_request)) => {
                Self::handle_status_request(
                    sequence_number,
                    request_tx,
                    service_name,
                    actions_registry,
                    start,
                )
                .await
            }
            Some(zelos_proto::actions::actions_response::Msg::Schema(schema_request)) => {
                Self::handle_schema_request(
                    sequence_number,
                    schema_request,
                    request_tx,
                    actions_registry,
                )
                .await
            }
            None => {
                tracing::warn!("Received ActionsResponse with no message");
                Ok(())
            }
        }
    }

    async fn handle_list_request(
        sequence_number: u64,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
        actions_registry: &Arc<ActionsRegistry>,
    ) -> Result<(), ActionsError> {
        tracing::debug!(
            "Handling list request from server (sequence={})",
            sequence_number
        );

        // Get the flat list of actions directly from the registry
        let actions_list = actions_registry.list()?;
        tracing::debug!(
            "Sending {} actions to server: {:?}",
            actions_list.len(),
            actions_list
        );

        let list_response = ListResponse {
            actions: actions_list,
        };

        let request = ActionsRequest {
            sequence_number,
            name: "client".to_string(),
            msg: Some(zelos_proto::actions::actions_request::Msg::List(
                list_response,
            )),
        };

        request_tx.send(request).map_err(|_| {
            ActionsError::ExecutionError("Failed to send list response".to_string())
        })?;

        tracing::debug!("Successfully sent list response to server");
        Ok(())
    }

    async fn handle_execute_request(
        sequence_number: u64,
        execute_request: ExecuteRequest,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
        actions_registry: &Arc<ActionsRegistry>,
        stream_token: &CancellationToken,
    ) -> Result<(), ActionsError> {
        tracing::debug!(
            "Server requested execution of action '{}' with params: {} (sequence: {})",
            execute_request.action,
            execute_request.params,
            sequence_number
        );

        // Malformed params must still yield a correlated error response carrying
        // the original sequence number, so the caller sees a failure instead of
        // timing out.
        let params: Value = match serde_json::from_str(&execute_request.params) {
            Ok(params) => params,
            Err(e) => {
                tracing::error!("Failed to parse params JSON: {}", e);
                let response = ExecuteResponse {
                    action: execute_request.action.clone(),
                    result: error_json(format!("Failed to parse params: {}", e)),
                    status: Some(ExecuteStatus::Error.into()),
                };
                Self::send_execute_response(sequence_number, response, request_tx)?;
                return Ok(());
            }
        };

        // Resolve timeout: request timeout_ms > action default_timeout_ms > no timeout.
        // A timeout_ms of 0 is treated as absent (matching the Go SDK), falling
        // back to the action's default timeout.
        let timeout = execute_request
            .timeout_ms
            .filter(|&ms| ms > 0)
            .or_else(|| {
                actions_registry
                    .get(&execute_request.action)
                    .ok()
                    .and_then(|a| a.default_timeout_ms())
            })
            .map(|ms| Duration::from_millis(ms as u64));

        // Spawn execution as a separate task so the stream loop remains responsive
        // to list/status/schema requests while actions run concurrently.
        let request_tx = request_tx.clone();
        let actions_registry = actions_registry.clone();
        let stream_token = stream_token.clone();
        tokio::spawn(async move {
            let action = execute_request.action;
            let execute_fut = actions_registry.execute(&action, params);

            // Abort promptly if the stream is torn down while the action runs.
            let timed = tokio::select! {
                biased;
                _ = stream_token.cancelled() => {
                    tracing::debug!("Action '{}' cancelled due to stream teardown", action);
                    return;
                }
                timed = async {
                    match timeout {
                        Some(duration) => tokio::time::timeout(duration, execute_fut)
                            .await
                            .map_err(|_| duration),
                        None => Ok(execute_fut.await),
                    }
                } => timed,
            };

            let response = match timed {
                Err(duration) => {
                    tracing::warn!("Action '{}' timed out after {:?}", action, duration);
                    ExecuteResponse {
                        action: action.clone(),
                        result: error_json(format!(
                            "Action timed out after {}ms",
                            duration.as_millis()
                        )),
                        status: Some(ExecuteStatus::Error.into()),
                    }
                }
                Ok(Ok(result)) => {
                    tracing::debug!(
                        "Action '{}' executed successfully, result: {:?}",
                        action,
                        result
                    );
                    result.into_execute_response(&action)
                }
                Ok(Err(e)) => {
                    tracing::error!("Action '{}' execution failed: {}", action, e);
                    ExecuteResponse {
                        action: action.clone(),
                        result: error_json(e.to_string()),
                        status: Some(ExecuteStatus::Error.into()),
                    }
                }
            };

            if let Err(e) = Self::send_execute_response(sequence_number, response, &request_tx) {
                tracing::error!("Failed to send execute response: {}", e);
            }
        });

        Ok(())
    }

    /// Send an [`ExecuteResponse`] back to the server, correlated by sequence number.
    fn send_execute_response(
        sequence_number: u64,
        response: ExecuteResponse,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
    ) -> Result<(), ActionsError> {
        let request = ActionsRequest {
            sequence_number,
            name: "client".to_string(),
            msg: Some(zelos_proto::actions::actions_request::Msg::Execute(
                response,
            )),
        };
        request_tx.send(request).map_err(|_| {
            ActionsError::ExecutionError("Failed to send execute response".to_string())
        })
    }

    async fn handle_status_request(
        sequence_number: u64,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
        service_name: &str,
        actions_registry: &Arc<ActionsRegistry>,
        start: Instant,
    ) -> Result<(), ActionsError> {
        tracing::debug!("Handling status request from server");

        let status_response = StatusResponse {
            json: serde_json::json!({
                "uptime_seconds": start.elapsed().as_secs(),
                "actions_count": actions_registry.list().unwrap_or_default().len(),
            })
            .to_string(),
        };

        let request = ActionsRequest {
            sequence_number,
            name: service_name.to_string(),
            msg: Some(zelos_proto::actions::actions_request::Msg::Status(
                status_response,
            )),
        };

        request_tx.send(request).map_err(|_| {
            ActionsError::ExecutionError("Failed to send status response".to_string())
        })?;

        Ok(())
    }

    async fn handle_schema_request(
        sequence_number: u64,
        schema_request: zelos_proto::actions::SchemaRequest,
        request_tx: &mpsc::UnboundedSender<ActionsRequest>,
        actions_registry: &Arc<ActionsRegistry>,
    ) -> Result<(), ActionsError> {
        tracing::debug!(
            "Handling schema request from server for action: {}",
            schema_request.action
        );

        // Parse current_values if provided. Malformed input yields a correlated
        // error schema response carrying the original sequence number rather than
        // returning early and leaving the caller to time out.
        let current_values = if schema_request.current_values.is_empty() {
            None
        } else {
            match serde_json::from_str(&schema_request.current_values) {
                Ok(value) => Some(value),
                Err(e) => {
                    tracing::error!("Failed to parse current_values JSON: {}", e);
                    let schema_response = zelos_proto::actions::SchemaResponse {
                        action: schema_request.action.clone(),
                        action_schema: error_json(format!(
                            "Failed to parse current_values: {}",
                            e
                        )),
                        ui_schema: "{}".to_string(),
                        default_timeout_ms: None,
                    };
                    let request = ActionsRequest {
                        sequence_number,
                        name: "client".to_string(),
                        msg: Some(zelos_proto::actions::actions_request::Msg::Schema(
                            schema_response,
                        )),
                    };
                    request_tx.send(request).map_err(|_| {
                        ActionsError::ExecutionError("Failed to send schema response".to_string())
                    })?;
                    return Ok(());
                }
            }
        };

        // Get the action-defined default timeout (if any)
        let default_timeout_ms = actions_registry
            .get(&schema_request.action)
            .ok()
            .and_then(|a| a.default_timeout_ms());

        let schema_response = match actions_registry
            .get_schema_json(&schema_request.action, current_values)
        {
            Ok((action_schema, ui_schema, _description)) => zelos_proto::actions::SchemaResponse {
                action: schema_request.action.clone(),
                action_schema,
                ui_schema,
                default_timeout_ms,
            },
            Err(e) => {
                tracing::error!(
                    "Schema request failed for action {}: {}",
                    schema_request.action,
                    e
                );
                // Return an error schema response
                zelos_proto::actions::SchemaResponse {
                    action: schema_request.action.clone(),
                    action_schema: error_json(e.to_string()),
                    ui_schema: "{}".to_string(),
                    default_timeout_ms: None,
                }
            }
        };

        let request = ActionsRequest {
            sequence_number,
            name: "client".to_string(),
            msg: Some(zelos_proto::actions::actions_request::Msg::Schema(
                schema_response,
            )),
        };

        request_tx.send(request).map_err(|_| {
            ActionsError::ExecutionError("Failed to send schema response".to_string())
        })?;

        tracing::debug!(
            "Successfully sent schema response to server for action: {}",
            schema_request.action
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::error_json;
    use serde_json::Value;

    #[test]
    fn error_json_escapes_hostile_strings() {
        let hostile = "boom \"quoted\" \\backslash\\ and\nnewline\ttab";
        let payload = error_json(hostile);

        // The produced payload is valid JSON with the message preserved verbatim.
        let parsed: Value = serde_json::from_str(&payload).expect("error payload is valid JSON");
        assert_eq!(parsed["error"], Value::String(hostile.to_string()));
    }
}
