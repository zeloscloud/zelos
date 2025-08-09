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

    let (client, client_task) = TracePublishClient::new(
        router.clone(),
        TracePublishClientConfig {
            url: url.clone(),
            ..Default::default()
        },
    );
    tokio::spawn(client_task);
    client.wait_until_connected(Duration::from_secs(5)).await?;

    // Build schema asynchronously and then emit using async helpers
    let source = TraceSource::new("async-schema-demo", router.sender());
    let evt = source
        .build_event("telemetry")
        .add_u64_field("seq", None)
        .add_f64_field("value", None)
        .build_async()
        .await?;

    evt.build()
        .try_insert_u64("seq", 1)?
        .try_insert_f64("value", 42.0)?
        .emit_async()
        .await?;

    Ok(())
}
