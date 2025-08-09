use std::time::Duration;

use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());

    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    // Custom client configuration
    let config = TracePublishClientConfig {
        url: url.clone(),
        batch_size: 256,
        batch_timeout: Duration::from_millis(50),
        reconnect_delay: Duration::from_millis(500),
    };

    let (client, client_task) = TracePublishClient::new(router.clone(), config);
    tokio::spawn(client_task);

    client.wait_until_connected(Duration::from_secs(5)).await?;
    println!("Connected to agent at {url}");

    // Emit a short burst of events
    let source = TraceSource::new("publish-config-demo", router.sender());
    let evt = source
        .build_event("sample")
        .add_u64_field("n", None)
        .build()?;

    for n in 1..=10u64 {
        evt.build().try_insert_u64("n", n)?.emit()?;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(())
}
