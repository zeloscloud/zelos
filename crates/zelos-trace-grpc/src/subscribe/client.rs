use anyhow::Result;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::Streaming;
use zelos_proto::trace::{
    SubscribeCommand, SubscribeRequest, SubscribeResponse, UnsubscribeCommand, subscribe_request,
    trace_subscribe_client,
};

pub struct TraceSubscribeClient {
    /// The sender for subscribe requests.
    req_sender: Sender<SubscribeRequest>,
}

impl TraceSubscribeClient {
    /// Create a new TraceSubscribeClient and connect to the given address.
    pub async fn new(
        sender: zelos_trace_types::ipc::Sender,
        cancellation_token: CancellationToken,
        address: String,
    ) -> Result<(Self, impl Future<Output = Result<()>>)> {
        // Connect to the gRPC server
        let mut client = trace_subscribe_client::TraceSubscribeClient::connect(address).await?;

        // Initialize a channel for sending subscribe requests
        let (req_sender, req_receiver) = tokio::sync::mpsc::channel(1);

        // Attempt to call the subscribe streaming method, exiting early if we fail
        let request_stream = tonic::Request::new(ReceiverStream::new(req_receiver));
        let resp = client.subscribe(request_stream).await?;

        // Run our task to forward from the response stream to the sender
        let future = Self::run(resp.into_inner(), sender.clone(), cancellation_token);

        Ok((Self { req_sender }, future))
    }

    /// Run a task to forward from the response stream to the sender
    async fn run(
        mut stream: Streaming<SubscribeResponse>,
        sender: zelos_trace_types::ipc::Sender,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        loop {
            tokio::select! {
                msg = stream.message() => {
                    match msg {
                        Ok(Some(response)) => {
                            // Forward the message to the router
                            let ipc = response.as_ipc()?;
                            for m in ipc {
                                sender.send_async(m).await?;
                            }
                        }
                        Ok(None) => {
                            // Stream ended
                            return Ok(());
                        }
                        Err(e) => {
                            // Error from the client
                            return Err(e.into());
                        }
                    }
                }
                _ = cancellation_token.cancelled() => return Ok(())
            }
        }
    }

    /// Send a subscribe command with the given filter and start time
    pub async fn subscribe(&self, filter: Option<String>, start_time: Option<i64>) -> Result<()> {
        self.req_sender
            .send(SubscribeRequest {
                cmd: Some(subscribe_request::Cmd::Subscribe(SubscribeCommand {
                    filter,
                    start_time,
                })),
            })
            .await?;

        Ok(())
    }

    /// Send an unsubscribe command with the given filter
    pub async fn unsubscribe(&self, filter: Option<String>) -> Result<()> {
        self.req_sender
            .send(SubscribeRequest {
                cmd: Some(subscribe_request::Cmd::Unsubscribe(UnsubscribeCommand {
                    filter,
                })),
            })
            .await?;

        Ok(())
    }

    /// Send a subscribe command with a blank filter
    pub async fn subscribe_all(&self) -> Result<()> {
        self.subscribe(None, None).await
    }

    /// Send an unsubscribe command with a blank filter
    pub async fn unsubscribe_all(&self) -> Result<()> {
        self.unsubscribe(None).await
    }
}
