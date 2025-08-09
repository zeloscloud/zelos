use std::{collections::HashMap, sync::LazyLock, time::Duration};

use anyhow::{Result, anyhow};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use zelos_trace::TraceRouter;
use zelos_trace_grpc::publish::{
    TracePublishClient, TracePublishClientConfig, TracePublishService,
};
use zelos_trace_types::{
    Value,
    ipc::{IpcMessage, IpcMessageWithId, Sender},
};

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to build runtime")
});

fn main() {
    // Configure tracing
    let filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|rust_log| rust_log.parse::<Targets>().ok())
        .unwrap_or(Targets::new().with_default(LevelFilter::ERROR));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();

    divan::main();
}

/// Benchmark data configurations
#[derive(Clone)]
struct BenchConfig {
    name: &'static str,
    num_segments: usize,
    events_per_segment: usize,
    fields_per_event: usize,
}

impl std::fmt::Display for BenchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} events, {} fields per event)",
            self.name,
            self.num_segments * self.events_per_segment,
            self.fields_per_event
        )
    }
}

const BENCH_CONFIGS: &[BenchConfig] = &[
    BenchConfig {
        name: "small",
        num_segments: 1,
        events_per_segment: 1_000,
        fields_per_event: 5,
    },
    BenchConfig {
        name: "medium",
        num_segments: 1,
        events_per_segment: 100_000,
        fields_per_event: 10,
    },
    BenchConfig {
        name: "large",
        num_segments: 1,
        events_per_segment: 1_000_000,
        fields_per_event: 20,
    },
];

async fn run_server(
    sender: Sender,
    shutdown: CancellationToken,
) -> Result<(String, JoinHandle<Result<()>>)> {
    // Create a tokio tcp listener on a random port and get it's address
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    // Spawn our grpc server
    let task_server = tokio::spawn(async move {
        Server::builder()
            .add_service(TracePublishService::new(sender, shutdown.clone()).server())
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown.cancelled())
            .await?;
        Ok(())
    });

    Ok((format!("grpc://{}", addr), task_server))
}

fn create_test_message(name: String, seq: u64, fields_per_event: usize) -> IpcMessageWithId {
    let mut fields = HashMap::new();
    fields.insert("seq".to_string(), Value::UInt64(seq));

    // Add additional fields based on configuration
    for i in 0..fields_per_event.saturating_sub(1) {
        fields.insert(format!("field_{}", i), Value::Int32(i as i32));
    }

    let message = IpcMessage::TraceEvent(zelos_trace_types::ipc::TraceEvent {
        time_ns: zelos_trace::time::now_time_ns(),
        name,
        fields,
    });

    IpcMessageWithId {
        segment_id: Uuid::now_v7(),
        source_name: "bench".to_string(),
        msg: message,
    }
}

mod router {
    use super::*;

    #[divan::bench(args = BENCH_CONFIGS, sample_count = 5)]
    fn publish(config: &BenchConfig) {
        RUNTIME.block_on(async move {
            let shutdown = CancellationToken::new();
            let (server_router, fut_server_router) = TraceRouter::new(shutdown.clone());
            tokio::spawn(fut_server_router);
            let (server_url, task_server) = run_server(server_router.sender(), shutdown.clone())
                .await
                .expect("Failed to run server");

            // Give the server time to start
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Configure the client
            let client_config = TracePublishClientConfig {
                url: server_url,
                batch_size: 1000,
                batch_timeout: Duration::from_millis(100),
                reconnect_delay: Duration::from_millis(100),
            };

            // Create the router
            let (router, fut_router) = TraceRouter::new(shutdown.clone());
            tokio::spawn(fut_router);

            // Create the client
            let (client, fut_publish) = TracePublishClient::new(router.clone(), client_config);
            let task_publish = tokio::spawn(fut_publish);

            // Wait until the client connects
            client
                .wait_until_connected(Duration::from_millis(100))
                .await
                .expect("Failed to connect");

            // Calculate total messages to send
            let total_messages = config.num_segments * config.events_per_segment;

            // Send messages
            let sender = router.sender();
            let name = format!("bench-{}", config.name);
            let fields_per_event = config.fields_per_event;
            let send_task = tokio::spawn(async move {
                for i in 0..total_messages {
                    let message = create_test_message(name.clone(), i as u64, fields_per_event);
                    sender
                        .send_async(message)
                        .await
                        .map_err(|e| anyhow!("Send error: {}", e))?;
                }
                Ok::<(), anyhow::Error>(())
            });
            send_task.await.unwrap().unwrap();

            // Wait for all messages to be confirmed
            loop {
                let status = client.last_publish_status().await;
                if let Some(status) = status {
                    if status.successful_messages >= total_messages as u64 {
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            // Shutdown
            task_publish.abort();
            shutdown.cancel();
            task_server.await.unwrap().unwrap();
        });
        divan::black_box(())
    }
}

mod direct {
    use tokio_stream::StreamExt;
    use tonic::Request;

    use super::*;

    #[divan::bench(args = BENCH_CONFIGS, sample_count = 5)]
    fn publish(config: &BenchConfig) {
        RUNTIME.block_on(async move {
            // Create a server that forwards messages to a black hole channel as fast as possible
            let shutdown = CancellationToken::new();
            let (sender, receiver) = flume::bounded(1024);
            tokio::spawn(async move {
                while let Ok(_msg) = receiver.recv_async().await {
                    // Do nothing
                }
            });
            let (server_url, task_server) = run_server(sender, shutdown.clone())
                .await
                .expect("Failed to run server");

            // Give the server time to start
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Attempt to connect to the grpc server
            tracing::info!("Trace client connecting to {}", &server_url);
            let mut client =
                zelos_proto::trace::trace_publish_client::TracePublishClient::connect(server_url)
                    .await
                    .expect("Failed to connect to publish service");

            // Create our message stream
            let (sender, receiver) = flume::bounded::<IpcMessageWithId>(1024);
            let stream = receiver
                .into_stream()
                .chunks_timeout(1000, Duration::from_millis(100))
                .map(|m| zelos_proto::trace::PublishRequest {
                    trace_messages: m.into_iter().map(|msg| msg.into()).collect(),
                });

            // Publish the stream to the server
            // Call our rpc to publish to the server
            let response = client
                .publish(Request::new(stream))
                .await
                .expect("Failed to establish publish stream");

            // Calculate total messages to send
            let total_messages = config.num_segments * config.events_per_segment;

            // Send messages
            let name = format!("bench-{}", config.name);
            let fields_per_event = config.fields_per_event;
            let send_task = tokio::spawn(async move {
                for i in 0..total_messages {
                    let message = create_test_message(name.clone(), i as u64, fields_per_event);
                    sender
                        .send_async(message)
                        .await
                        .map_err(|e| anyhow!("Send error: {}", e))?;
                }
                Ok::<(), anyhow::Error>(())
            });
            send_task.await.unwrap().unwrap();

            // Wait for all messages to be confirmed
            let mut response_stream = response.into_inner();
            while let Some(Ok(msg)) = response_stream.next().await {
                if let Some(status) = msg.status {
                    if status.successful_messages >= total_messages as u64 {
                        break;
                    }
                }
            }

            // Shutdown
            shutdown.cancel();
            task_server.await.unwrap().unwrap();
        });
        divan::black_box(())
    }
}
