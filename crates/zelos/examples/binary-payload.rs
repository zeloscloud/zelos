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

    let source = TraceSource::new("binary-demo", router.sender());
    let evt = source
        .build_event("blob")
        .add_string_field("name", None)
        .add_binary_field("data", None)
        .build()?;

    let payload: Vec<u8> = (0..=255u16).map(|b| b as u8).collect();
    evt.build()
        .try_insert_string("name", "bytes_0_255".into())?
        .try_insert_binary("data", payload)?
        .emit()?;

    println!("emitted binary payload event");
    Ok(())
}
