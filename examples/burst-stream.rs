use std::time::{Duration, Instant};
use tokio::time;
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

    let source = TraceSource::new("events", router.sender());

    let start_event = source
        .build_event("event_start")
        .add_string_field("trigger", None)
        .add_i64_field("timestamp_ms", None)
        .build()?;

    let sample_event = source
        .build_event("event_sample")
        .add_f64_field("value", None)
        .add_u32_field("index", None)
        .build()?;

    let end_event = source
        .build_event("event_end")
        .add_f64_field("duration_ms", None)
        .add_u32_field("sample_count", None)
        .build()?;

    loop {
        time::sleep(Duration::from_secs(5)).await;

        let start_time = Instant::now();
        start_event
            .build()
            .try_insert_string("trigger", "threshold_exceeded".to_string())?
            .try_insert_i64("timestamp_ms", start_time.elapsed().as_millis() as i64)?
            .emit()?;

        for i in 0..100u32 {
            let value = i as f64 * 0.1;
            sample_event
                .build()
                .try_insert_f64("value", value)?
                .try_insert_u32("index", i)?
                .emit()?;
        }

        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        end_event
            .build()
            .try_insert_f64("duration_ms", duration_ms)?
            .try_insert_u32("sample_count", 100)?
            .emit()?;
    }
}
