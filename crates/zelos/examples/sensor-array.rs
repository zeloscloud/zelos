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

    let source = TraceSource::new("sensor-array", router.sender());

    // Define schema with 16 float32 fields
    let mut builder = source.build_event("array");
    for i in 0..16u8 {
        builder = builder.add_f32_field(&format!("sensor_{i}"), None);
    }
    let array_event = builder.build()?;

    // Emit a few arrays
    for t in 0..10u32 {
        let mut b = array_event.build();
        for i in 0..16u8 {
            let value = (t as f32) * 0.1 + (i as f32) * 0.01;
            b = b.try_insert_f32(&format!("sensor_{i}"), value)?;
        }
        b.emit()?;
    }

    Ok(())
}
