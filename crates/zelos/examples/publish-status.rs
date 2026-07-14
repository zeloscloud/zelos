use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Setup client and connect
    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    let (client, client_task) = TracePublishClient::new(
        router.clone(),
        TracePublishClientConfig {
            url: url.clone(),
            ..Default::default()
        },
    );
    tokio::spawn(client_task);

    client.wait_until_connected(Duration::from_secs(5)).await?;
    println!("Connected to {url}");

    // Print publish status updates for a short period
    let mut status_rx = client.publish_status().await;
    tokio::spawn(async move {
        loop {
            if status_rx.changed().await.is_err() {
                break;
            }
            if let Some(status) = status_rx.borrow().clone() {
                println!("Publish status: {:?}", status);
            }
        }
    });

    // Emit a few events so we can potentially observe status messages
    let source = TraceSource::new("publish-status-demo", router.sender());
    let evt = source
        .build_event("sample")
        .add_u64_field("n", None)
        .build()?;

    for n in 1..=5u64 {
        evt.build().try_insert_u64("n", n)?.emit()?;
    }

    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(())
}
