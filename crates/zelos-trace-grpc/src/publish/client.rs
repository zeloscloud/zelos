use std::{sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use tokio::{sync::watch, time::Instant};
use tokio_stream::StreamExt;
use tonic::{Code, Request};
use zelos_proto::trace::{
    PublishRequest, PublishStatus, trace_publish_client::TracePublishClient as GrpcClient,
};
use zelos_trace::TraceRouter;

use crate::connection_status::ConnectionStatus;

const DEFAULT_BATCH_SIZE: usize = 1000;
const DEFAULT_BATCH_TIMEOUT_MS: u64 = 100;
const DEFAULT_URL: &str = "grpc://localhost:2300";
const DEFAULT_RECONNECT_DELAY_MS: u64 = 1000;

#[derive(Debug, Clone)]
pub struct TracePublishClientConfig {
    /// URL of the trace publish service
    pub url: String,
    /// Maximum number of messages to batch together in a single request
    pub batch_size: usize,
    /// Maximum time to wait before sending a batch (even if not full)
    pub batch_timeout: Duration,
    /// Minimum delay between connection attempts
    pub reconnect_delay: Duration,
}

impl TracePublishClientConfig {
    pub fn new_with_url(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

impl Default for TracePublishClientConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
            batch_timeout: Duration::from_millis(DEFAULT_BATCH_TIMEOUT_MS),
            reconnect_delay: Duration::from_millis(DEFAULT_RECONNECT_DELAY_MS),
        }
    }
}

pub struct TracePublishClient {
    /// The config, retained for external visibility
    pub config: TracePublishClientConfig,

    /// The last connection status
    connection_status: watch::Receiver<ConnectionStatus>,

    /// The last publish status from the connection
    publish_status: watch::Receiver<Option<PublishStatus>>,
}

impl TracePublishClient {
    /// Create a new client with the provided config
    pub fn new(
        router: Arc<TraceRouter>,
        config: TracePublishClientConfig,
    ) -> (Self, impl Future<Output = Result<()>>) {
        let (tx_connection_status, connection_status) =
            watch::channel(ConnectionStatus::Disconnected);
        let (tx_publish_status, publish_status) = watch::channel(None);

        let client = Self {
            config: config.clone(),
            connection_status,
            publish_status,
        };
        let task = Self::run(router, config, tx_publish_status, tx_connection_status);

        (client, task)
    }

    /// Create a new client with default configuration
    pub fn new_with_default_config(
        router: Arc<TraceRouter>,
    ) -> (Self, impl Future<Output = Result<()>>) {
        Self::new(router, TracePublishClientConfig::default())
    }

    /// Connect to the endpoint specified in the config, returning with an error on failure.
    async fn connect(
        router: Arc<TraceRouter>,
        config: TracePublishClientConfig,
        tx_publish_status: &watch::Sender<Option<PublishStatus>>,
        tx_connection_status: &watch::Sender<ConnectionStatus>,
    ) -> Result<()> {
        // Attempt to connect to the grpc server
        tracing::info!("Trace client connecting to {}", &config.url);
        let mut client = GrpcClient::connect(config.url.clone())
            .await
            .map_err(|e| anyhow!("Failed to connect to publish service: {}", e))?;

        // Subscribe to all messages and get it back as a stream
        let stream = router
            .subscribe_all_blocking_stream()
            .await?
            .chunks_timeout(config.batch_size, config.batch_timeout)
            .map(|m| PublishRequest {
                trace_messages: m.into_iter().map(|msg| msg.into()).collect(),
            });

        // Call our rpc to publish to the server
        let response = client
            .publish(Request::new(stream))
            .await
            .map_err(|e| anyhow!("Failed to establish publish stream: {e}"))?;
        tracing::debug!("Successfully established new gRPC publish stream.");
        tx_connection_status.send(ConnectionStatus::Connected)?;

        // Process status messages sent back from the server
        let mut response_stream = response.into_inner();
        loop {
            match response_stream.message().await {
                Ok(Some(resp)) => {
                    if let Some(status) = resp.status {
                        tx_publish_status.send(Some(status))?;
                    }
                    tracing::trace!("Publish status: {:?}", tx_publish_status);
                }
                Ok(None) => {
                    // Stream closed normally
                    return Ok(());
                }
                Err(status) => {
                    return match status.code() {
                        Code::Ok => Ok(()),
                        _ => Err(anyhow!("Received error status: {}", status)),
                    };
                }
            }
        }
    }

    /// The main task loop, which attempts to reconnect repeatedly while waiting for reconnect_delay
    async fn run(
        router: Arc<TraceRouter>,
        config: TracePublishClientConfig,
        tx_publish_status: watch::Sender<Option<PublishStatus>>,
        tx_connection_status: watch::Sender<ConnectionStatus>,
    ) -> Result<()> {
        let mut last_connection_time: Instant;
        loop {
            // Attempt to connect
            last_connection_time = Instant::now();
            tx_connection_status.send(ConnectionStatus::Connecting)?;
            if let Err(e) = Self::connect(
                router.clone(),
                config.clone(),
                &tx_publish_status,
                &tx_connection_status,
            )
            .await
            {
                tracing::error!("Error forwarding trace events: {}", e);
                tx_connection_status.send(ConnectionStatus::Error)?;
            } else {
                tx_connection_status.send(ConnectionStatus::Disconnected)?;
            }

            // If our reconnect attempt was too recent, sleep until our reconnect delay is up
            let elapsed = last_connection_time.elapsed();
            if elapsed < config.reconnect_delay {
                let remaining = config.reconnect_delay - elapsed;
                tokio::time::sleep(remaining).await;
            }
        }
    }

    /// Gets a clone of the connection status receiver
    pub async fn connection_status(&self) -> watch::Receiver<ConnectionStatus> {
        self.connection_status.clone()
    }

    /// Gets the last value the connection status receiver has seen
    pub async fn last_connection_status(&self) -> ConnectionStatus {
        self.connection_status.borrow().clone()
    }

    /// Wait until connected or the timeout expires
    pub async fn wait_until_connected(&self, timeout: Duration) -> Result<()> {
        let mut connection_status = self.connection_status.clone();
        tokio::time::timeout(timeout, async move {
            loop {
                // Borrow and get the current value, this also updates the receiver if needed
                let status = *connection_status.borrow_and_update();

                // Check if we are already connected
                if status == ConnectionStatus::Connected {
                    return Ok(()); // We are connected, success
                }

                // If not connected, wait for the next status change.
                // This returns an error if the sender is dropped (client shut down).
                connection_status.changed().await?;
            }
        })
        .await // Await the timeout future
        // If the timeout occurs, map the Elapsed error to an anyhow error
        .map_err(|_| {
            anyhow!(
                "Timed out waiting for connection to become connected within {:?}",
                timeout
            )
        })?
    }

    /// Gets a clone of the publish status receiver
    pub async fn publish_status(&self) -> watch::Receiver<Option<PublishStatus>> {
        self.publish_status.clone()
    }

    /// Gets the last value the publish status receiver has seen
    pub async fn last_publish_status(&self) -> Option<PublishStatus> {
        self.publish_status.borrow().clone()
    }
}
