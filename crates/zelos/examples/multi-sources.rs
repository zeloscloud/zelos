use rand::Rng;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};

async fn stream_motor(source: TraceSource) -> anyhow::Result<()> {
    let telemetry = source
        .build_event("telemetry")
        .add_f64_field("rpm", Some("rpm".to_string()))
        .add_f64_field("torque", Some("Nm".to_string()))
        .build()?;

    let mut interval = tokio::time::interval(Duration::from_millis(10)); // 100 Hz
    let mut rng = rand::thread_rng();

    loop {
        interval.tick().await;
        telemetry
            .build()
            .try_insert_f64("rpm", 2000.0 + rng.gen_range(-100.0..100.0))?
            .try_insert_f64("torque", 50.0 + rng.gen_range(-5.0..5.0))?
            .emit()?;
    }
}

async fn stream_battery(source: TraceSource) -> anyhow::Result<()> {
    let status = source
        .build_event("status")
        .add_f64_field("voltage", Some("V".to_string()))
        .add_f64_field("current", Some("A".to_string()))
        .add_f64_field("soc", Some("%".to_string()))
        .build()?;

    let mut interval = tokio::time::interval(Duration::from_secs(1)); // 1 Hz
    let mut rng = rand::thread_rng();
    let mut soc: f64 = 85.0;

    loop {
        interval.tick().await;
        soc = if soc - 0.1 < 20.0 { 20.0 } else { soc - 0.1 };
        status
            .build()
            .try_insert_f64("voltage", 48.0 + rng.gen_range(-0.5..0.5))?
            .try_insert_f64("current", rng.gen_range(-10.0..50.0))?
            .try_insert_f64("soc", soc)?
            .emit()?;
    }
}

async fn stream_gps(source: TraceSource) -> anyhow::Result<()> {
    let position = source
        .build_event("position")
        .add_f64_field("lat", Some("deg".to_string()))
        .add_f64_field("lon", Some("deg".to_string()))
        .add_f64_field("alt", Some("m".to_string()))
        .build()?;

    let mut interval = tokio::time::interval(Duration::from_millis(100)); // 10 Hz
    let mut rng = rand::thread_rng();

    let base_lat = 37.4419;
    let base_lon = -122.1430;

    loop {
        interval.tick().await;
        position
            .build()
            .try_insert_f64("lat", base_lat + rng.gen_range(-0.001..0.001))?
            .try_insert_f64("lon", base_lon + rng.gen_range(-0.001..0.001))?
            .try_insert_f64("alt", 30.0 + rng.gen_range(-1.0..1.0))?
            .emit()?;
    }
}

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

    // Create sources
    let motor = TraceSource::new("motor", router.sender());
    let battery = TraceSource::new("battery", router.sender());
    let gps = TraceSource::new("gps", router.sender());

    tokio::select! {
        res = stream_motor(motor) => res?,
        res = stream_battery(battery) => res?,
        res = stream_gps(gps) => res?,
    }
    Ok(())
}
