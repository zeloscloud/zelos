use std::{future::Future, sync::Arc};

use anyhow::Result;
use tokio::{sync::oneshot, time::Instant};
use tokio_stream::{Stream, StreamExt};
use tokio_util::sync::CancellationToken;
use zelos_trace_types::ipc::{IpcMessageWithId, Receiver, Sender};

use crate::{
    MetadataOnlyStore, Store, TraceSink,
    sink::{TraceSinkHandle, TraceSinkHandleAllBlocking},
};

// TODO(tkeairns): Ground this constant into some relationship with # msgs/sec
pub const DEFAULT_CHANNEL_SIZE: usize = 1024;

/// Subscription requests sent to the router's main task
type SubscriptionRequest = (
    Box<dyn TraceSinkHandle>,
    oneshot::Sender<Result<Vec<IpcMessageWithId>>>,
);

/// Pub-sub router for trace data
pub struct TraceRouter {
    // Channel for broadcasting trace streams to subscribers
    sender: Sender,

    // Channel for subscription requests
    subscription_sender: flume::Sender<SubscriptionRequest>,
}

impl TraceRouter {
    /// Create a new trace router with the default metadata-only store
    pub fn new(
        cancellation_token: CancellationToken,
    ) -> (Arc<Self>, impl Future<Output = Result<()>>) {
        let store = Arc::new(MetadataOnlyStore::new());
        Self::new_with_store(store, cancellation_token)
    }

    /// Create a new trace router with a specific store implementation
    pub fn new_with_store(
        store: Arc<dyn Store>,
        cancellation_token: CancellationToken,
    ) -> (Arc<Self>, impl Future<Output = Result<()>>) {
        // Initialize the channel for receiving trace streams.
        let (sender, receiver) = flume::bounded(DEFAULT_CHANNEL_SIZE);

        // Initialize the channel for subscription requests
        let (subscription_sender, subscription_receiver) = flume::bounded(1);

        let router = TraceRouter {
            sender,
            subscription_sender,
        };

        // Spawn the router's main task
        let run = TraceRouter::run(receiver, subscription_receiver, store, cancellation_token);

        (Arc::new(router), run)
    }

    async fn forward_message(
        store: &Arc<dyn Store>,
        sinks: &mut Vec<Box<dyn TraceSinkHandle>>,
        msg: IpcMessageWithId,
    ) {
        // Update the store
        if let Err(e) = store.update(&msg) {
            tracing::error!("Error while updating the store: {}", e);
        }

        // Forward this message to all subscribers
        let mut closed_sinks = Vec::new();
        {
            metrics::gauge!("router_sinks", "task" => "router").set(sinks.len() as f64);

            for (idx, sink) in sinks.iter().enumerate() {
                if let Err(e) = sink.send_async(&msg).await {
                    tracing::trace!("Error when sending on sink: {}", e);
                    // If we have an error here, this means that the sink is no longer
                    // available, so we add it to the list of sinks to remove
                    closed_sinks.push(idx);
                }
            }
        }

        // Remove all closed sinks
        if !closed_sinks.is_empty() {
            // Sort in reverse order so we can remove from highest index to lowest
            // without affecting the validity of the remaining indices
            closed_sinks.sort_unstable_by(|a, b| b.cmp(a));

            for idx in closed_sinks {
                // Remove the sink at the index
                sinks.remove(idx);
            }
        }
    }

    async fn handle_subscribe(
        store: &Arc<dyn Store>,
        sinks: &mut Vec<Box<dyn TraceSinkHandle>>,
        handle: Box<dyn TraceSinkHandle>,
        sub_response_sender: oneshot::Sender<Result<Vec<IpcMessageWithId>>>,
    ) {
        sinks.push(handle);

        if let Err(e) = sub_response_sender.send(store.metadata_as_ipc()) {
            tracing::error!("Failed to send metadata to new subscriber: {:?}", e);
        }
    }

    async fn run(
        receiver: Receiver,
        subscription_receiver: flume::Receiver<SubscriptionRequest>,
        store: Arc<dyn Store>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        // Construct task-local state
        let mut sinks = Vec::new();

        loop {
            tokio::select! {
                // Handle subscription requests
                sub_req = subscription_receiver.recv_async() => {
                    match sub_req {
                        Ok((handle, sub_response_sender)) => {
                            Self::handle_subscribe(&store, &mut sinks, handle, sub_response_sender).await;
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                msg = receiver.recv_async() => {
                    let msg = msg?;

                    // Update our metrics
                    metrics::counter!("messages_received", "task" => "router").increment(1);
                    metrics::gauge!("receiver_len", "task" => "router").set(receiver.len() as f64);

                    // Update our state and forward
                    let start = Instant::now();
                    TraceRouter::forward_message(&store, &mut sinks, msg).await;
                    let elapsed = start.elapsed();

                    metrics::histogram!("update_store_duration_ns", "task" => "router")
                        .record(elapsed.as_nanos() as f64);
                }
                _ = cancellation_token.cancelled() => {
                    tracing::debug!("Shutting down...");

                    // Drain the receiver
                    let start = Instant::now();
                    let mut count: usize = 0;
                    for msg in receiver.drain() {
                        TraceRouter::forward_message(&store, &mut sinks, msg).await;
                        count += 1;
                    }
                    let elapsed = start.elapsed();

                    tracing::debug!("Shut down complete, took {:?} processed {} messages", elapsed, count);
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }

    /// Subscribe to all data, applying backpressure when needed
    pub async fn subscribe_all_blocking(&self) -> Result<(Receiver, Vec<IpcMessageWithId>)> {
        let (handle, receiver) = TraceSinkHandleAllBlocking::new();
        let (sub_response_sender, sub_response_receiver) = oneshot::channel();

        self.subscription_sender
            .send_async((Box::new(handle), sub_response_sender))
            .await
            .map_err(|_| anyhow::anyhow!("Router subscription channel closed"))?;

        // Wait for the router to return metadata
        let metadata = sub_response_receiver
            .await
            .map_err(|_| anyhow::anyhow!("Response channel closed"))??;

        Ok((receiver, metadata))
    }

    /// Subscribe to trace streams
    pub async fn subscribe(&self) -> Result<(TraceSink, Receiver, Vec<IpcMessageWithId>)> {
        let (sink, receiver, handle) = TraceSink::new();
        let (sub_response_sender, sub_response_receiver) = oneshot::channel();

        self.subscription_sender
            .send_async((Box::new(handle), sub_response_sender))
            .await
            .map_err(|_| anyhow::anyhow!("Router subscription channel closed"))?;

        // Wait for the router to return metadata
        let metadata = sub_response_receiver
            .await
            .map_err(|_| anyhow::anyhow!("Response channel closed"))??;

        Ok((sink, receiver, metadata))
    }

    pub async fn subscribe_all_blocking_stream(
        &self,
    ) -> Result<impl Stream<Item = IpcMessageWithId> + use<>> {
        let (receiver, metadata) = self.subscribe_all_blocking().await?;
        Ok(tokio_stream::iter(metadata).chain(receiver.into_stream()))
    }

    pub async fn subscribe_stream(
        &self,
    ) -> Result<(TraceSink, impl Stream<Item = IpcMessageWithId> + use<>)> {
        let (sink, receiver, metadata) = self.subscribe().await?;
        let stream = tokio_stream::iter(metadata).chain(receiver.into_stream());
        Ok((sink, stream))
    }
}
