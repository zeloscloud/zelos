use rand::Rng;
use std::time::Duration;
use tokio::time;
use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Connect publisher client to the running agent
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

    // Define schema
    let source = TraceSource::new("sensors", router.sender());
    let temperature_event = source
        .build_event("temperature")
        .add_f64_field("value", Some("°C".to_string()))
        .build()?;

    // Stream at 1 Hz
    let mut interval = time::interval(Duration::from_secs(1));
    let mut rng = rand::thread_rng();

    loop {
        interval.tick().await;
        let temperature = 20.0 + rng.gen_range(-2.0..2.0);
        temperature_event
            .build()
            .try_insert_f64("value", temperature)?
            .emit()?;
    }
}
