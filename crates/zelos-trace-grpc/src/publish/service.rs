use std::pin::Pin;

use tokio::{sync::mpsc, time::Duration};
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status, Streaming};
use zelos_proto::trace::{
    trace_publish_server::{TracePublish, TracePublishServer},
    PublishRequest, PublishResponse, PublishStatus,
};
use zelos_trace_types::ipc::Sender;

pub struct TracePublishService {
    sender: Sender,
    cancellation_token: CancellationToken,
}

impl TracePublishService {
    pub fn new(sender: Sender, cancellation_token: CancellationToken) -> Self {
        Self {
            sender,
            cancellation_token,
        }
    }

    pub fn server(self) -> TracePublishServer<Self> {
        TracePublishServer::new(self)
    }
}

/// Forward all of the messages in req to the router sender, returning a grpc error on failure
async fn forward_request_messages(req: PublishRequest, sender: &Sender) -> Result<usize, Status> {
    let count = req.trace_messages.len();
    for msg in req.trace_messages {
        // If try_into fails, we have an invalid proto message that we cannot understand
        let ipc = msg
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("Error converting message: {}", e)))?;
        // If send_async fails, the router has shutdown and we cannot send any more messages
        sender
            .send_async(ipc)
            .await
            .map_err(|e| Status::unavailable(format!("Error sending message: {}", e)))?;
    }
    Ok(count)
}

#[tonic::async_trait]
impl TracePublish for TracePublishService {
    type PublishStream =
        Pin<Box<dyn Stream<Item = Result<PublishResponse, Status>> + Send + 'static>>;

    async fn publish(
        &self,
        request: Request<Streaming<PublishRequest>>,
    ) -> Result<Response<Self::PublishStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<PublishResponse, Status>>(1);

        // Spawn our task to forward messages from the request to the router
        let mut stream = request.into_inner();
        let router_sender = self.sender.clone();
        let shutdown = self.cancellation_token.clone();
        tokio::spawn(async move {
            let mut msg_count = 0;

            // Send a heartbeat message to the client once per second
            let mut status_interval = tokio::time::interval(Duration::from_secs(1));
            status_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            // Run our main processing loop
            loop {
                tokio::select! {
                    msg = stream.message() => {
                        match msg {
                            Ok(Some(req)) => {
                                match forward_request_messages(req, &router_sender).await {
                                    Ok(count) => msg_count += count as u64,
                                    Err(e) => {
                                        // We had an error forwarding the request, attempt to send that error to the
                                        // client and then exit
                                        tracing::error!("Error forwarding trace messages: {}", e);
                                        let _ = tx.try_send(Err(e));
                                        return;
                                    }
                                }
                            }
                            Ok(None) => {
                                // Client closed the stream, shutdown
                                return;
                            }
                            Err(err) => {
                                // We had an error receiving a message from the client, shutdown
                                tracing::error!("Error from client: {}", err);
                                return;
                            }
                        }
                    }
                    _ = status_interval.tick() => {
                        // Send a heartbeat message to the client
                        // NOTE(jbott): we close the connection on failure rather than sending an error because there is
                        // no way to recover.
                        if tx.try_send(Ok(PublishResponse { status: Some(PublishStatus {
                            total_messages: msg_count,
                            successful_messages: msg_count,
                            failed_messages: 0,
                            last_error: "".to_string(),
                        }) })).is_err() {
                            // Client disconnected, exit
                            return;
                        }
                    }
                    _ = shutdown.cancelled() => {
                        // Server shutting down, inform the client and exit
                        let _ = tx.try_send(Err(Status::unavailable("Server shutting down".to_string())));
                        return;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
