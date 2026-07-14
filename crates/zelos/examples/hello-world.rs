use std::time::Duration;

use tokio_util::sync::CancellationToken;
use zelos_trace::TraceRouter;
use zelos_trace::TraceSource;
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configuration
    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    println!("Connecting to Zelos agent at: {}", url);

    // Set up the router and cancellation token (using Arc like the working example)
    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    // Set up the publish client to connect to the agent
    let config = TracePublishClientConfig {
        url: url.clone(),
        ..Default::default()
    };
    let (client, client_task) = TracePublishClient::new(router.clone(), config);
    tokio::spawn(client_task);

    // Wait for the client to connect
    client
        .wait_until_connected(Duration::from_secs(5))
        .await
        .expect("Failed to connect to agent");
    println!("Connected to agent at {}", url);

    // Create a TraceSource and register a 'hello' event with a 'message' field
    // Using u64 field like the working example
    let source = TraceSource::new("hello-world-example", router.sender());
    let hello_event = source
        .build_event("hello")
        .add_u64_field("count", None)
        .add_u64_field("timestamp", Some("ns".to_string()))
        .build()
        .expect("Failed to register hello event");

    // Publish a single hello message
    println!("Publishing hello message...");
    if let Err(e) = hello_event
        .build()
        .try_insert_u64("count", 1)
        .and_then(|b| b.try_insert_u64("timestamp", zelos_trace::time::now_time_ns() as u64))
        .and_then(|b| b.emit())
    {
        eprintln!("Failed to emit hello event: {e}");
        return Err(e.into());
    }

    println!("Successfully published hello message!");
    println!("Check your Zelos agent/collector to see the data.");

    // Give a moment for the message to be sent
    tokio::time::sleep(Duration::from_millis(1000)).await;

    Ok(())
}
