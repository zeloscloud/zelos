use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::{TraceRouter, TraceSource};
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};
use zelos_trace_types::Value;

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

    let source = TraceSource::new("state-machine", router.sender());
    let state_event = source
        .build_event("state")
        .add_u8_field("current", None)
        .add_u8_field("previous", None)
        .add_f64_field("transition_time_ms", None)
        .build()?;

    // Add value tables for readable state names
    source.add_value_table(
        "state",
        "current",
        [
            (Value::UInt8(0), "IDLE".to_string()),
            (Value::UInt8(1), "INIT".to_string()),
            (Value::UInt8(2), "RUNNING".to_string()),
            (Value::UInt8(3), "ERROR".to_string()),
        ]
        .into_iter(),
    )?;
    source.add_value_table(
        "state",
        "previous",
        [
            (Value::UInt8(0), "IDLE".to_string()),
            (Value::UInt8(1), "INIT".to_string()),
            (Value::UInt8(2), "RUNNING".to_string()),
            (Value::UInt8(3), "ERROR".to_string()),
        ]
        .into_iter(),
    )?;

    // Emit some sample transitions
    let transitions = [
        (0u8, 1u8, 12.3),
        (1u8, 2u8, 5.4),
        (2u8, 3u8, 1.1),
        (3u8, 0u8, 20.7),
    ];
    for (previous, current, ms) in transitions {
        state_event
            .build()
            .try_insert_u8("current", current)?
            .try_insert_u8("previous", previous)?
            .try_insert_f64("transition_time_ms", ms)?
            .emit()?;
    }

    Ok(())
}
