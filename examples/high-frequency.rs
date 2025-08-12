use std::time::{Duration, Instant};
use tokio::time::{interval, MissedTickBehavior};
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

    let source = TraceSource::new("high_freq", router.sender());
    let data_event = source
        .build_event("data")
        .add_f64_field("value", Some("V".to_string()))
        .build()?;

    // 1 kHz = 1ms period
    let mut tick = interval(Duration::from_millis(1));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let start = Instant::now();
    loop {
        tick.tick().await;
        let elapsed = start.elapsed().as_secs_f64();
        let value = (2.0 * std::f64::consts::PI * 100.0 * elapsed).sin();
        data_event.build().try_insert_f64("value", value)?.emit()?;
    }
}
