use std::time::Duration;

use clap::Parser;
use tokio_util::sync::CancellationToken;
use zelos_trace::TraceRouter;
use zelos_trace::TraceSource;
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[derive(Parser, Debug, Clone)]
struct Args {
    /// URL of the trace publish service (agent)
    #[clap(short, long, default_value = "grpc://127.0.0.1:2300")]
    url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    // Set up the publish client to connect to the agent
    let config = TracePublishClientConfig {
        url: args.url.clone(),
        ..Default::default()
    };
    let (client, client_task) = TracePublishClient::new(router.clone(), config);
    tokio::spawn(client_task);

    client
        .wait_until_connected(Duration::from_secs(5))
        .await
        .expect("Failed to connect to agent");
    println!("Connected to agent at {}", args.url);

    // Register a 'hello' event with a 'count' field
    let source = TraceSource::new("hello-world-pub", router.sender());
    let hello_event = source
        .build_event("hello")
        .add_u64_field("count", None)
        .build()
        .expect("Failed to register hello event");

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    println!("Publishing hello event every second. Press Ctrl+C to exit.");

    let shutdown_token = cancellation_token.clone();
    let shutdown = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl_c");
        println!("\nReceived Ctrl+C, shutting down...");
        shutdown_token.cancel();
    });

    let mut count: u64 = 0;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                count += 1;
                if let Err(e) = hello_event.build().try_insert_u64("count", count).and_then(|b| b.emit()) {
                    eprintln!("Failed to emit hello event: {e}");
                } else {
                    println!("Published hello event! count={}", count);
                }
            }
            _ = cancellation_token.cancelled() => {
                break;
            }
        }
    }
    let _ = shutdown.await;
}
