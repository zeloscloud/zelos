use std::time::Duration;
use tokio_util::sync::CancellationToken;
use zelos_trace::TraceRouter;
use zelos_trace::TraceSource;
use zelos_trace_grpc::publish::{TracePublishClient, TracePublishClientConfig};
use zelos_trace_types::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cancellation_token = CancellationToken::new();
    let (router, router_task) = TraceRouter::new(cancellation_token.clone());
    tokio::spawn(router_task);

    // Connect publish client to the running agent
    let url = std::env::var("ZELOS_URL").unwrap_or_else(|_| "grpc://127.0.0.1:2300".to_string());
    let (client, client_task) = TracePublishClient::new(
        router.clone(),
        TracePublishClientConfig {
            url: url.clone(),
            ..Default::default()
        },
    );
    tokio::spawn(client_task);
    client.wait_until_connected(Duration::from_secs(5)).await?;

    let source = TraceSource::new("status-demo", router.sender());

    // Define an event with a status code and message
    let status_evt = source
        .build_event("status")
        .add_u8_field("status_code", None)
        .add_string_field("detail", None)
        .build()?;

    // Provide a value table for status_code to improve downstream readability
    source.add_value_table(
        "status",
        "status_code",
        [
            (Value::UInt8(0), "idle".to_string()),
            (Value::UInt8(1), "busy".to_string()),
            (Value::UInt8(2), "error".to_string()),
        ]
        .into_iter(),
    )?;

    status_evt
        .build()
        .try_insert_u8("status_code", 1)?
        .try_insert_string("detail", "processing request".into())?
        .emit()?;

    println!("emitted status event with value table metadata");
    Ok(())
}
