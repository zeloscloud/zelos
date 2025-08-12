use std::time::Duration;
use tokio::time;
use tokio_util::sync::CancellationToken;
use zelos_trace::{time::now_time_ns, TraceRouter, TraceSource};
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

    let source = TraceSource::new("replay", router.sender());
    let measurement = source
        .build_event("measurement")
        .add_f64_field("value", None)
        .build()?;

    // Method 1: automatic timestamp
    measurement.build().try_insert_f64("value", 1.0)?.emit()?;

    // Method 2: specific timestamp
    let specific_time_ns: i64 = 1_699_564_234_567_890_123;
    measurement
        .build()
        .try_insert_f64("value", 2.0)?
        .emit_at(specific_time_ns)?;

    // Method 3: calculated timestamp (1 minute ago)
    let past_time_ns = now_time_ns() - (60 * 1_000_000_000);
    measurement
        .build()
        .try_insert_f64("value", 3.0)?
        .emit_at(past_time_ns)?;

    // Method 4: synchronized timestamps in a loop
    let sync_offset_ns: i64 = 123_456; // example offset
    let mut interval = time::interval(Duration::from_millis(100));
    for _ in 0..5 {
        interval.tick().await;
        let timestamp_ns = now_time_ns() + sync_offset_ns;
        measurement
            .build()
            .try_insert_f64("value", 4.2)?
            .emit_at(timestamp_ns)?;
    }

    Ok(())
}
